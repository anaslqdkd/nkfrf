use std::future::pending;
use std::rc::Rc;

use gtk4::ffi::gtk_label_new;
use gtk4::gdk::Display;
use gtk4::{prelude::*, Button, CssProvider, Label, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk4::{Application, ApplicationWindow};
use glib::{timeout_add_local, MainContext, Priority};
// use zbus::blocking::connection;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use zbus::{connection, interface, zvariant};

struct NotificationService {
    count: u64,
    sender: glib::Sender<NotificationData>,
}
struct NotificationData {
    summary: String,
    body: String,
}
// TODO: implement the queue
#[interface(name = "org.freedesktop.Notifications")]
impl NotificationService {
    fn say_hello(&mut self, name: &str) -> String {
        self.count += 1;
        format!("Hello {}! I have been called {} times.", name, self.count)
    }
    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        (
            "NotifDaemon", 
            "Me",          
            "0.0",         
            "0.0",         
        )
    }
    fn notify( &self, app_name: &str, replaces_id: u32, app_icon: &str, summary: &str, body: &str, actions: Vec<String>, hints: std::collections::HashMap<String, zvariant::Value>, expire_timeout: i32,) -> u32 {
        println!("the notification is app_name{}, body{}", app_name, body);
        let summary = summary.to_string();
        let body = body.to_string();
        let icon = app_icon.to_string();
        let notification = NotificationData {
            summary: summary.to_string(),
            body: body.to_string()
        };
        self.sender.send(notification).expect("Pb with the send notif");
        1
    }
}
fn show_notification_popup(summary: &str, body: &str, window: Rc<ApplicationWindow>){
    println!("the notif is {}, {}", summary, body);
    let label = Label::new(Some(body));
    label.add_css_class("white-label");
    if let Some(container) = window.child(){
        if let Some(box_container) = container.downcast_ref::<gtk4::Box>() {
            box_container.append(&label);
        }
    }
    window.present();
    timeout_add_local(std::time::Duration::from_secs(10), move || {
        window.hide();
        glib::ControlFlow::Continue
    });

}
#[tokio::main] 
async fn main() {
	let app = Application::new(
		Some("com.example.asyncgtk"),
		Default::default(),
	);

	app.connect_activate(|app| {
		let window = Rc::new(ApplicationWindow::builder()
			.application(app)
			.title("Async GTK Example")
			// .default_width(300)
			// .default_height(100)
			.build());
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_default_size(300, 100);
        window.set_opacity(0.95);
        let anchors = [
            (Edge::Left, false),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ];
        for (anchor, state) in anchors {
            window.set_anchor(anchor, state);
        }
        let provider = CssProvider::new();
        provider.load_from_path("style.css");
        let display = Display::default().expect("Could not connect to display");
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.add_css_class("container-bg");

        // let button = Button::with_label("Idk");
        // button.add_css_class("my-button");
        // button.remove_css_class("text-button");

        window.set_child(Some(&container));

        window.set_margin(Edge::Right, 40);
        window.set_margin(Edge::Top, 20);
        window.show();
        // NOTE: deprecated but works
        let (sender, receiver) = glib::MainContext::channel::<NotificationData>(glib::Priority::DEFAULT);
        receiver.attach(None, move |notification| {
            let win_clone = window.clone();
            show_notification_popup(&notification.summary, &notification.body, win_clone);
            glib::ControlFlow::Continue
        });

		// window.show();

		// Spawn an async task on the GTK main loop
		MainContext::default().spawn_local(async move {

			println!("Done!");
            let greeter = NotificationService {count:0, sender};
            let _conn = connection::Builder::session().expect("Pb here")
                .name("org.freedesktop.Notifications").expect("Pb name")
                .serve_at("/org/freedesktop/Notifications", greeter).expect("Pb servve at")
                .build()
                .await;
            pending::<()>().await;
            // Ok(())
		});
	});

	app.run();
}

