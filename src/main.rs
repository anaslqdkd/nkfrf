use anyhow::Ok;
use clap::{Parser, Subcommand, ValueEnum};
use glib::{ControlFlow, MainContext, Priority, timeout_add_local};
use gtk4::gdk::Display;
use gtk4::{Application, ApplicationWindow};
use gtk4::{Button, CssProvider, Label, STYLE_PROVIDER_PRIORITY_APPLICATION, prelude::*};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::app::App;
use crate::app_state::{AppState, Message};
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
    command: Option<CliCommands>,
}
#[derive(Debug, Subcommand)]
enum CliCommands {
    Show,
    Close,
    DoNotDisturbEnable,
}
// TODO: manage dbus app calls ex close
// TODO: implement configuration options
// TODO: do not disturb mode

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
    });
    let dbus_client_ = dbus_client::DbusClient::init().await.expect("buu");
    let cli = Cli::parse();
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder().application(app).build();
        let provider = CssProvider::new();
        provider.load_from_path("style.css");
        gtk4::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display"),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        window.hide();
    });

    if let Some(cmd) = &cli.command {
        match cmd {
            CliCommands::Show => dbus_client_.show_nc().await?,
            CliCommands::Close => dbus_client_.close_nc().await?,
            CliCommands::DoNotDisturbEnable => dbus_client_.enable_dnd().await?,
        };
    }
    let app_handler_clone = app_handler.clone();
    let app_handler_clone_ = app_handler.clone();
    receiver.attach(None, move |notification| {
        let message = Message::NotificationReceived(notification);
        app_handler_clone.handle_message(message);
        glib::ControlFlow::Continue
    });
    request_receiver.attach(None, move |server_request| {
        let message = Message::from(server_request);
        app_handler_clone_.handle_message(message);
        glib::ControlFlow::Continue
    });

    app.run();

    Ok(())
}
