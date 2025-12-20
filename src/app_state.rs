use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use gtk4::{Application, ApplicationWindow, prelude::WidgetExt};

use crate::{app::Command, dbus::{self, Notification}, notification_window::NotificationWindow};

const MAX_ACTIVE: usize = 1;
#[derive(Debug, Clone)]
pub struct NotificationPopup{
    pub window: ApplicationWindow,
    pub notification_id: u32,
    pub notification: Notification,
}
impl NotificationPopup{
    pub fn new(notification: Notification, app: &Application) -> Self{
        Self {
            window:NotificationWindow::new(&notification.summary, &notification.body).build(app),
            notification_id: notification.id,
            notification,
        }
    }
    pub fn show_popup(&self){
        self.window.show();
    }
}

pub struct AppState{
    pub queue: VecDeque<dbus::Notification>,
    pub active_popups: Vec<NotificationPopup>, 
}
impl AppState{
    pub fn new() -> Self{
        Self {
            queue: VecDeque::new(),
            active_popups: Vec::new(),
        }
    } 
    pub fn update(&mut self, message:Message) -> Command{
        match message {
            Message::NotificationReceived(notification) => {
                println!("AppState: The notification was received");
                let notification_clone = notification.clone();
                self.queue.push_back(notification);
                if self.can_show_popup(){
                    if let Some(enqued_notification) = self.queue.pop_front(){
                        Command::ShowPopup(enqued_notification)
                    }else{
                        Command::None
                    }
                }else{
                    println!("Cannot show notification");
                    Command::None
                }
            }
            Message::PopupExpired(id) => {
                println!("PopupExpired: ID {}", id);
                Command::HidePopup(id)
            }
            Message::SendNextNotification => {
                println!("Trying sending the next notification");
                if let Some(next_notification) = self.queue.pop_front() {
                    println!(" Showing next notification from queue: {}", next_notification.summary);
                    Command::ShowPopup(next_notification)
                } else {
                    Command::None
                }
            }
            _ => {
                println!("In the other use cases");
                Command::None
            }
        }
    }
    pub fn add_active_popup(&mut self, popup: NotificationPopup){
        println!("The popup is added in active popups with id {}", popup.notification_id);
        self.active_popups.push(popup);
    }
    pub fn remove_popup(&mut self, id: u32) {
        self.active_popups.retain(|p| p.notification_id != id);
    }
    pub fn can_show_popup(&self) -> bool{
        println!("the number of active popup is {}", self.active_popups.len());
        let res = self.active_popups.len() < MAX_ACTIVE;
        println!("Can show window {}", res);
        println!("the number of active popups is {} and MAX_ACTIVE is {}", self.active_popups.len(), MAX_ACTIVE);
        res
    }
}


#[derive(Debug, Clone)]
pub enum Message {
    NotificationReceived(Notification),
    // PopupClicked(u32),
    PopupExpired(u32),
    SendNextNotification,
    // ToggleDoNotDisturb,
    // CloseAllNotifications,
}
