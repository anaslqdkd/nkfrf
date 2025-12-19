use std::rc::Rc;

use anyhow::Error;
use zbus::Connection;
use zbus::{Message, Proxy, interface, zvariant};

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
        )
        .await?;
        let dbus_client = Self { proxy };
        Ok(dbus_client)
    }
    pub async fn show_nc(&self) -> Result<(), anyhow::Error> {
        self.proxy.call_method("ShowNc", &()).await?;
        Ok(())
    }
    pub async fn close_nc(&self) -> Result<(), anyhow::Error> {
        self.proxy.call_method("CloseNc", &()).await?;
        Ok(())
    }
}
