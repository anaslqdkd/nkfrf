use std::future::pending;
use std::rc::Rc;
use std::time::Duration;

// use zbus::blocking::Connection;
use zbus::Connection;
use zbus::zvariant::{OwnedValue, Value};
use zbus::{connection, interface, zvariant, Proxy};
#[derive(Debug, Clone)]
pub struct Hints {
    pub urgency: u8,
}
#[derive(Debug, Clone)]
pub struct NotificationData {
    pub summary: String,
    pub body: String,
    pub icon: String,
    pub actions: Vec<String>,
    pub hints: Hints,
    pub expire_timeout: i32,
}
#[derive(Debug, Clone)]
pub enum ServerRequestItem{
    OpenNC,
}
#[derive(Debug, Clone)]
pub struct NotificationService {
    count: u64,
    sender: glib::Sender<NotificationData>,
    request_sender: glib::Sender<ServerRequestItem>,
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
        let hints_struct = Hints {
            urgency: match hints.get("urgency") {
                Some(Value::U8(u)) => *u,
                _ => 1, 
            },
        };
        let mut duration = 5000;
        if expire_timeout != -1{
            duration = expire_timeout;
        }

        let notification = NotificationData {
            summary: summary,
            body: body,
            icon: icon,
            actions: actions,
            expire_timeout: duration,
            hints: hints_struct,
            
        };
        println!("the hints are: {:#?}", hints);
        self.sender.send(notification).expect("Pb with the send notif");
        1
    }
    async fn show_nc(&self) -> zbus::fdo::Result<()> {
        println!("$$$$");
        self.request_sender.send(ServerRequestItem::OpenNC).unwrap();
        Ok(())
    }
}
pub async fn run(sender: glib::Sender<NotificationData>, request_sender: glib::Sender<ServerRequestItem> ) -> anyhow::Result<()> {
    println!("Done!");

    let greeter = NotificationService {count:0, sender, request_sender };
    let _conn = connection::Builder::session().expect("Pb here")
        .name("org.freedesktop.Notifications").expect("Pb name")
        .serve_at("/org/freedesktop/Notifications", greeter).expect("Pb servve at")
        .build()
        .await;

    pending::<()>().await;
    Ok(())

}
