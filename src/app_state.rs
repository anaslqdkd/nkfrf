use std::collections::VecDeque;

use gtk4::{Application, ApplicationWindow, prelude::WidgetExt};
use gtk4_layer_shell::LayerShell;

use crate::{TOP_MARGIN, WINDOW_HEIGHT, app::Command, dbus::{self, Notification}, notification_window::NotificationWindow};

const MAX_ACTIVE: usize = 3;
#[derive(Debug, Clone)]
pub struct NotificationPopup{
    pub window: ApplicationWindow,
    pub notification_id: u32,
    pub notification: Notification,
}
impl NotificationPopup{
    pub fn new(notification: Notification, app: &Application, order_nb: u32) -> Self{
        Self {
            window:NotificationWindow::new(notification.clone()).build(app, order_nb),
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
    pub dnd: bool,
}
impl AppState{
    pub fn new() -> Self{
        Self {
            queue: VecDeque::new(),
            active_popups: Vec::new(),
            dnd: false,
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
                    // FIXME: after detoggling dnd and if there is more that one notification in the queue the notification is shown one by one
                    if self.can_show_popup(){
                        println!(" Showing next notification from queue: {}", next_notification.summary);
                        Command::ShowPopup(next_notification)
                    }else{
                        Command::None
                    }
                } else {
                    Command::None
                }
            }
            Message::RedrawWindows => {
                println!("Redrawing windows");
                Command::RedrawWindows
            }
            Message::ToggleDoNotDisturb => {
                println!("Toggling do not disturb");
                Command::ToggleDoNotDisturb
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
    pub fn toggle_dnd(&mut self){
        // TODO: check if there are notifications from the queue to show after disabling toggle mode
        self.dnd = !self.dnd;
    }
    pub fn can_show_popup(&self) -> bool{
        if self.dnd{
            false
        }else{
            println!("the number of active popup is {}", self.active_popups.len());
            let res = self.active_popups.len() < MAX_ACTIVE;
            println!("Can show window {}", res);
            println!("the number of active popups is {} and MAX_ACTIVE is {}", self.active_popups.len(), MAX_ACTIVE);
            res

        }
    }
    pub fn active_popups_nb(&mut self) -> u32{
        let length = self.active_popups.len() as u32;
        println!("the number of active_popups is {} ", length);
        length
    }
    pub fn redraw_windows(&mut self){
        for (index, popup_obj) in self.active_popups.iter().enumerate(){
            let y_offset = (index as i32) * WINDOW_HEIGHT + TOP_MARGIN;
            popup_obj.window.set_margin(gtk4_layer_shell::Edge::Top, y_offset);
        }
    }
}


#[derive(Debug, Clone)]
pub enum Message {
    NotificationReceived(Notification),
    RedrawWindows,
    // PopupClicked(u32),
    PopupExpired(u32),
    SendNextNotification,
    ToggleDoNotDisturb,
    ShowNotificationCenter,
    CloseNotificationCenter,
    // CloseAllNotifications,
}

