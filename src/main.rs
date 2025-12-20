use anyhow::Ok;
use clap::{Parser, Subcommand, ValueEnum};
use dbus::ServerRequestItem;
use glib::{ControlFlow, MainContext, Priority, timeout_add_local};
use gtk4::gdk::Display;
use gtk4::subclass::window;
use gtk4::{Application, ApplicationWindow};
use gtk4::{Button, CssProvider, Label, STYLE_PROVIDER_PRIORITY_APPLICATION, prelude::*};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use pango;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::future::pending;
use std::rc::Rc;
use zbus::zvariant::{OwnedValue, Value};
use zbus::{Connection, proxy};

use crate::app::App;
use crate::app_state::{AppState, Message};
use crate::notification_window::NotificationWindow;
mod dbus;
mod dbus_client;
mod notification_window;
mod app_state;
mod app;

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
    Close,
}
// TODO: implement critical/normal priorities
// TODO: manage dbus app calls ex close


fn show_notification_popup(
    queue: &Rc<RefCell<VecDeque<dbus::Notification>>>,
    app: &Application,
    active_notifications: &Rc<RefCell<i32>>,
    active_windows: &WindowList,
) {
    if *active_notifications.borrow() >= MAX_ACTIVE {
        return;
    }
    let mut q = queue.borrow_mut();
    if let Some(notification) = q.pop_front() {
        // is_showing.set(true);
        let summary: &str = &notification.summary;
        let body: &str = &notification.body;
        let icon: &str = &notification.icon;
        let actions: &Vec<String> = &notification.actions;
        let expire_timeout: i32 = notification.expire_timeout;
        let hints: u8 = notification.hints.urgency;
        // println!("In the if statement with notification {:#?}", notification);
        // println!("actions = {:?}", actions);
        // let window = draw_window(summary, body, app, *active_notifications.borrow_mut());
        let window = NotificationWindow::new(summary, body).build(app);
        window.show();

        *active_notifications.borrow_mut() += 1;
        active_windows.borrow_mut().push(window.clone());
        let queue = queue.clone();
        let window = window.clone();
        let app_clone = app.clone();
        let active_notifications = active_notifications.clone();
        let active_windows = active_windows.clone();
        let duration = expire_timeout as u64;

        timeout_add_local(std::time::Duration::from_millis(duration), move || {
            // println!("The time is up");
            // println!("Queue: {:?}", queue.borrow());
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
        let y_offset = index * WINDOW_HEIGHT + TOP_MARGIN;
        win.set_margin(Edge::Top, y_offset);
        index += 1;
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (sender, receiver) =
        glib::MainContext::channel::<dbus::Notification>(glib::Priority::DEFAULT);
    let (request_sender, request_receiver) =
        glib::MainContext::channel::<dbus::ServerRequestItem>(glib::Priority::DEFAULT);
    let (notification_sender, notification_receiver) =
        glib::MainContext::channel::<dbus::Notification>(glib::Priority::DEFAULT);
    let app = Application::new(Some("com.example.asyncgtk"), Default::default());
    let app_handler = App::new(app.clone());
    gtk4::init()?;

    tokio::spawn(async move {
        dbus::run(sender, request_sender, notification_sender)
            .await
            .unwrap();
        // TODO: create a notification store object and use methods to add the notifications each time directly from main
    });
    let dbus_client_ = dbus_client::DbusClient::init().await.expect("buu");
    let cli = Cli::parse();
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder().application(app).build();
        window.hide();
    });

    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Show => dbus_client_.show_nc().await?,
            Commands::Close => dbus_client_.close_nc().await?,
        };
    }
    // TODO: implement ids for notifications instead
    // let mut app_state = AppState::new();

    let app_clone = app.clone();
    let queue = Rc::new(RefCell::new(VecDeque::<dbus::Notification>::new()));
    let active_notifications = Rc::new(RefCell::new(0));
    let active_windows: WindowList = Rc::new(RefCell::new(Vec::new()));
    let active_windows = active_windows.clone();
    let app_handler_clone = app_handler.clone();
    receiver.attach(None, move |notification| {
        let message = Message::NotificationReceived(notification);
        app_handler_clone.handle_message(message);
        // app_state.update(message);
        // queue.borrow_mut().push_back(notification.clone());
        // show_notification_popup(&queue, &app_clone, &active_notifications, &active_windows);
        glib::ControlFlow::Continue
    });

    app.run();

    Ok(())
}
