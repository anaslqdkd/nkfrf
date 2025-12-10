use std::rc::Rc;

use anyhow::Error;
use zbus::{interface, zvariant, Message, Proxy};
use zbus::Connection;

pub struct DbusClient {
    proxy: Proxy<'static>,
}

impl DbusClient {
    pub async fn init() -> Result<Self, anyhow::Error> {
        let connection = Connection::session().await?;
        let proxy = Proxy::new(
            &connection,
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
        ).await?;
        let dbus_client = Self {
            proxy: proxy
        };
        Ok(dbus_client)
    }
    pub async fn show_nc(&self) -> Result<(), anyhow::Error> {
        println!("in the show nc function in dbus_client");
        self.proxy.call_method("ShowNc", &()).await?;
        Ok(())
    }

}

