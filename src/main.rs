use std::cell::{Cell, RefCell};
use std::future::pending;
use std::rc::Rc;
use std::time::SystemTime;
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

const TOP_MARGIN: i32 = 20;
const WINDOW_HEIGHT: i32 = 70;
const WINDOW_WIDTH: i32 = 270;
const MAX_ACTIVE: i32 = 5;

type WindowList = Rc<RefCell<Vec<ApplicationWindow>>>;

struct NotificationService {
    count: u64,
    sender: glib::Sender<NotificationData>,
}

#[derive(Debug, Clone)]
struct NotificationData {
    summary: String,
    body: String,
}
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
fn draw_window (summary: &str, body: &str, app: &Application, stack_number: i32) -> ApplicationWindow{
    println!("in the draw window function");
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Async GTK Example")
        .default_width(300)
        .default_height(100)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    // TODO: make it adapt to size based on the display
    window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT);
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

    let y_offset = stack_number*WINDOW_HEIGHT + TOP_MARGIN;
    window.set_margin(Edge::Right, 20);
    window.set_margin(Edge::Top, y_offset);
    let label_body = Label::new(Some(body));
    let label_summary = Label::new(Some(summary));
    label_body.add_css_class("label-body");
    label_body.set_wrap_mode(pango::WrapMode::WordChar);
    label_body.set_wrap(true); 
    label_body.set_max_width_chars(40);
    label_body.set_xalign(0.0);
    label_body.set_yalign(0.0);
    label_summary.set_xalign(0.0);
    label_summary.set_yalign(0.0);

    label_summary.add_css_class("label-summary");
    if let Some(container) = window.child(){
        if let Some(box_container) = container.downcast_ref::<gtk4::Box>() {
            while let Some(child) = box_container.first_child() {
                box_container.remove(&child);
            }
            box_container.append(&label_summary);
            box_container.append(&label_body);
        }
    }
    window

}
fn show_notification_popup( queue: &Rc<RefCell<VecDeque<NotificationData>>>, is_showing: &Rc<Cell<bool>>, app: &Application, active_notifications: &Rc<RefCell<i32>>, active_windows: &WindowList){
    if *active_notifications.borrow() >= MAX_ACTIVE {
        return;
    }
    println!("in show notification popup");
    let mut q = queue.borrow_mut();
    if let Some(notification) = q.pop_front(){
        // is_showing.set(true);
        let summary: &str = &notification.summary;
        let body: &str = &notification.body;
        println!("In the if statement with summary {} body {}", summary,body);
        let window = draw_window(summary, body, app, *active_notifications.borrow_mut());
        *active_notifications.borrow_mut() += 1;
        window.show();
        active_windows.borrow_mut().push(window.clone());
        let queue = queue.clone();
        let is_showing = is_showing.clone();
        let window = window.clone();
        let app_clone = app.clone();
        let active_notifications = active_notifications.clone();
        let active_windows = active_windows.clone();
        timeout_add_local(std::time::Duration::from_secs(10), move || {
            println!("The time is up");
            println!("Queue: {:?}", queue.borrow());
            *active_notifications.borrow_mut() -= 1;
            active_windows.borrow_mut().retain(|w| !w.eq(&window));
            window.hide();
            redraw_windows(&active_windows);
            show_notification_popup(&queue, &is_showing, &app_clone, &active_notifications, &active_windows);
            glib::ControlFlow::Break
        });
    }
}
    //     // TODO: spaw windows until there is notification and until there are items in the queue
    //     // TODO: implement critical/normal priorities
fn redraw_windows(windows: &WindowList) {
    let mut index: i32 = 0;

    for win in windows.borrow().iter() {
        let y_offset = index*WINDOW_HEIGHT + TOP_MARGIN;
        win.set_margin(Edge::Top, y_offset);
        index += 1;
    }
}

#[tokio::main] 
async fn main() {
	let app = Application::new(
		Some("com.example.asyncgtk"),
		Default::default(),
	);

	app.connect_activate(|app| {
        let queue = Rc::new(RefCell::new(VecDeque::<NotificationData>::new()));
        let queue_for_receiver = queue.clone();
        let active_notifications = Rc::new(RefCell::new(0));
        let window = Rc::new(ApplicationWindow::builder()
            .application(app)
            .title("Async GTK Example")
            .default_width(300)
            .default_height(100)
            .build());
            window.init_layer_shell();
            window.set_layer(Layer::Overlay);
            // TODO: make it adapt to size based on the display
            window.set_default_size(270, 70);
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

        window.set_margin(Edge::Right, 20);
        window.set_margin(Edge::Top, 20);
        // NOTE: deprecated but works
        let (sender, receiver) = glib::MainContext::channel::<NotificationData>(glib::Priority::DEFAULT);
        let is_showing = Rc::new(Cell::new(false));
        let is_showing_clone = is_showing.clone();
        let app_clone = app.clone();
        let active_windows: WindowList = Rc::new(RefCell::new(Vec::new()));
        let active_windows = active_windows.clone();
        receiver.attach(None, move |notification| {
            println!("in the receiver attach");
            queue_for_receiver.borrow_mut().push_back(notification.clone());
            show_notification_popup(&queue_for_receiver, &is_showing_clone.clone(), &app_clone, &active_notifications, &active_windows);
            // if !(is_showing.get()){
            // }
            glib::ControlFlow::Continue
        });

		MainContext::default().spawn_local(async move {

			println!("Done!");
            let greeter = NotificationService {count:0, sender};
            let _conn = connection::Builder::session().expect("Pb here")
                .name("org.freedesktop.Notifications").expect("Pb name")
                .serve_at("/org/freedesktop/Notifications", greeter).expect("Pb servve at")
                .build()
                .await;

            pending::<()>().await;
		});

    });
	app.run();
}

