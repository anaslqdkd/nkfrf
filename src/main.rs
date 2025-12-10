use anyhow::Ok;
use dbus::ServerRequestItem;
use gtk4::subclass::window;
use gtk4::{Application, ApplicationWindow};
use clap::{Parser, Subcommand, ValueEnum};
use std::cell::{Cell, RefCell};
use std::future::pending;
use std::rc::Rc;
use zbus::zvariant::{OwnedValue, Value};
use zbus::{Connection, proxy};
use std::collections::{HashMap, VecDeque};
use pango;
use gtk4::gdk::Display;
use gtk4::{prelude::*, Button, CssProvider, Label, STYLE_PROVIDER_PRIORITY_APPLICATION};
use glib::{timeout_add_local, ControlFlow, MainContext, Priority};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
mod dbus;
mod dbus_client;

const TOP_MARGIN: i32 = 20;
const WINDOW_HEIGHT: i32 = 70;
const WINDOW_WIDTH: i32 = 270;
const MAX_ACTIVE: i32 = 5;

type WindowList = Rc<RefCell<Vec<ApplicationWindow>>>;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Debug, Subcommand)]
enum Commands {
    Show,
}
// TODO: implement critical/normal priorities

fn show_notification_popup( queue: &Rc<RefCell<VecDeque<dbus::NotificationData>>>, app: &Application, active_notifications: &Rc<RefCell<i32>>, active_windows: &WindowList){
    if *active_notifications.borrow() >= MAX_ACTIVE {
        return;
    }
    println!("in show notification popup");
    let mut q = queue.borrow_mut();
    if let Some(notification) = q.pop_front(){
        // is_showing.set(true);
        let summary: &str = &notification.summary;
        let body: &str = &notification.body;
        let icon: &str = &notification.icon;
        let actions: &Vec<String> = &notification.actions;
        let expire_timeout: i32 = notification.expire_timeout;
        let hints: u8 = notification.hints.urgency;
        println!("In the if statement with notification {:#?}", notification);
        println!("actions = {:?}", actions);
        let window = draw_window(summary, body, app, *active_notifications.borrow_mut());
        *active_notifications.borrow_mut() += 1;
        window.show();
        active_windows.borrow_mut().push(window.clone());
        let queue = queue.clone();
        let window = window.clone();
        let app_clone = app.clone();
        let active_notifications = active_notifications.clone();
        let active_windows = active_windows.clone();
        let duration = expire_timeout as u64;

        timeout_add_local(std::time::Duration::from_millis(duration), move || {
            println!("The time is up");
            println!("Queue: {:?}", queue.borrow());
            *active_notifications.borrow_mut() -= 1;
            active_windows.borrow_mut().retain(|w| !w.eq(&window));
            window.hide();
            redraw_windows(&active_windows);
            show_notification_popup(&queue, &app_clone, &active_notifications, &active_windows);
            glib::ControlFlow::Break
        });
    }
}
fn redraw_windows(windows: &WindowList) {
    let mut index: i32 = 0;

    for win in windows.borrow().iter() {
        let y_offset = index*WINDOW_HEIGHT + TOP_MARGIN;
        win.set_margin(Edge::Top, y_offset);
        index += 1;
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


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (sender, receiver) = glib::MainContext::channel::<dbus::NotificationData>(glib::Priority::DEFAULT);
    let (request_sender, request_receiver) = glib::MainContext::channel::<dbus::ServerRequestItem>(glib::Priority::DEFAULT);

    tokio::spawn(async move {
        dbus::run(sender, request_sender).await.unwrap();
    });
    let dbus_client_ = dbus_client::DbusClient::init().await.expect("buu");
    let cli = Cli::parse();
    let app = Application::new(
        Some("com.example.asyncgtk"),
        Default::default(),
    );
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder().application(app).build();
        window.hide();
    });


    if let Some(cmd) = &cli.command {
        match cmd {
            // Commands::Show => show_nc()?,
            Commands::Show => dbus_client_.show_nc().await?,
        };
    } 

    let app_clone = app.clone();
    let queue = Rc::new(RefCell::new(VecDeque::<dbus::NotificationData>::new()));
    let active_notifications = Rc::new(RefCell::new(0));
    let active_windows: WindowList = Rc::new(RefCell::new(Vec::new()));
    let active_windows = active_windows.clone();
    receiver.attach(None, move |notification| {
        println!("in the receiver attach");
        queue.borrow_mut().push_back(notification.clone());
        show_notification_popup(&queue, &app_clone, &active_notifications, &active_windows);
        glib::ControlFlow::Continue
    });
    let app_ = app.clone();
    request_receiver.attach(None, move |server_request_item| {
        println!("The receiver received something");
        match server_request_item {
            ServerRequestItem::OpenNC => {Some(test(&app_));
                glib::ControlFlow::Continue
            }
        }
    });

    
    app.run();

    Ok(())
}
fn test(app: &Application) -> Result<(), anyhow::Error>{
    // TODO: continue this function
    let window = ApplicationWindow::builder().application(app).build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_default_size(300, 600);
    window.set_opacity(0.5);
    let anchors = [
        (Edge::Left, false),
        (Edge::Right, true),
        (Edge::Top, true),
        (Edge::Bottom, false),
    ];
    for (anchor, state) in anchors {
        window.set_anchor(anchor, state);
    }
    window.set_margin(Edge::Right, 20);
    window.set_margin(Edge::Top, 20);
    let provider = CssProvider::new();
    provider.load_from_path("style.css");
    let display = Display::default().expect("Could not connect to display");
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    container.add_css_class("nc-bg");
    window.set_child(Some(&container));
    window.show();
    println!("++++");
    Ok(())
}
