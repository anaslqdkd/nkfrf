use std::cell::{Cell, RefCell};
use std::future::pending;
use std::rc::Rc;
use notify_rust::Notification;
use std::collections::VecDeque;
use pango;

use gtk4::ffi::gtk_label_new;
use gtk4::gdk::Display;
use gtk4::{prelude::*, Button, CssProvider, Label, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk4::{Application, ApplicationWindow};
use glib::{timeout_add_local, MainContext, Priority};
// use zbus::blocking::connection;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use zbus::{connection, interface, zvariant};
use lipsum::lipsum;

struct NotificationService {
    count: u64,
    sender: glib::Sender<NotificationData>,
}

#[derive(Debug, Clone)]
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
fn show_notification_popup(summary: &str, body: &str, window: Rc<ApplicationWindow>, is_showing: Rc<Cell<bool>>){
    println!("the notif is {}, {}", summary, body);
    let label = Label::new(Some(body));
    label.add_css_class("white-label");
    label.set_wrap_mode(pango::WrapMode::WordChar);
    label.set_wrap(true); // Enable wrapping
    label.set_max_width_chars(40);
    label.set_xalign(0.0);
    label.set_yalign(0.0);
    if let Some(container) = window.child(){
        if let Some(box_container) = container.downcast_ref::<gtk4::Box>() {
            while let Some(child) = box_container.first_child() {
                box_container.remove(&child);
            }
            box_container.append(&label);
        }
    }
    window.present();
    glib::MainContext::default().spawn_local({
        let window = window.clone();
        async move {
            window.show(); 
            glib::timeout_future(std::time::Duration::from_secs(10)).await;
            window.hide(); 
            is_showing.set(false);
        }
    });

    // timeout_add_local(std::time::Duration::from_secs(10), {
    //     let window = window.clone();
    //     move || {
    //         window.hide();
    //         glib::ControlFlow::Break
    //     }
    // });
    // timeout_add_local(std::time::Duration::from_secs(10), move || {
    //     window.hide();
    //     glib::ControlFlow::Continue
    // });

}
#[tokio::main] 
async fn main() {
	let app = Application::new(
		Some("com.example.asyncgtk"),
		Default::default(),
	);
    // let mut deque: VecDeque<NotificationData> = VecDeque::new();

	app.connect_activate(|app| {
        let queue = Rc::new(RefCell::new(VecDeque::<NotificationData>::new()));
        let queue_for_receiver = queue.clone();
		let window = Rc::new(ApplicationWindow::builder()
			.application(app)
			.title("Async GTK Example")
			.default_width(300)
			.default_height(100)
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


        window.set_child(Some(&container));

        window.set_margin(Edge::Right, 40);
        window.set_margin(Edge::Top, 20);
        // window.show();
        // NOTE: deprecated but works
        let (sender, receiver) = glib::MainContext::channel::<NotificationData>(glib::Priority::DEFAULT);
        let is_showing = Rc::new(Cell::new(false));
        let is_showing_clone = is_showing.clone();
        receiver.attach(None, move |notification| {
            let win_clone = window.clone();
            queue_for_receiver.borrow_mut().push_back(notification.clone());
            // println!("Queue: {:?}", queue_for_receiver.borrow());
            if !is_showing_clone.get() {
                println!("The notification is showing");
                is_showing_clone.set(true);
                if let Some(notification_) = queue_for_receiver.borrow_mut().pop_front(){
                    show_notification_popup(&notification_.summary, &notification.body, win_clone, is_showing_clone.clone());
                }
            }
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

            // show_notification_popup("Server Status", "Notification server is up and running.", window.clone());
            pending::<()>().await;
            // Ok(())
		});

    });
	app.run();
}

