use std::time::Duration;
use std::{error::Error, future::pending};
use glib::ControlFlow;
use notify_rust::Notification;
use std::sync::mpsc::{channel, Sender};
use tokio::time::Timeout;
use zbus::{connection, interface, zvariant};
use gtk4::gdk::Display;
use gtk4::gio::DBusConnection;
use gtk4::{self as gtk, Application, Button, CheckButton, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use serde_json::Value;
use zbus::{Connection, fdo};
use zbus::zvariant::OwnedValue;


struct NotificationService {
    count: u64,
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
        test();
        1
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let greeter = NotificationService { count: 0 };
    let _conn = connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", greeter)?
        .build()
        .await?;

    pending::<()>().await;

    Ok(())
}
fn test(){
    println!("Ta mère la pute");
}


