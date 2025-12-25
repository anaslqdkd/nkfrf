use std::{cell::RefCell, rc::Rc, time::Duration};

use glib::timeout_add_local;
use gtk4::{Application, prelude::GtkWindowExt};

use crate::{app_state::{AppState, Message, NotificationPopup}, dbus::Notification};

#[derive(Clone)]
pub struct App{
    pub app_gtk: Application,
    pub state: Rc<RefCell<AppState>>,
}
pub enum Command{
    ShowPopup(Notification),
    HidePopup(u32),
    RedrawWindows,
    None,
    ToggleDoNotDisturb,

}
impl App {
    pub fn new(app_gtk: Application) -> Self{
        Self {
            app_gtk,
            state: Rc::new(RefCell::new(AppState::new())),
        }
    }
    
    pub fn handle_message(&self, message: Message){
        let command = self.state.borrow_mut().update(message);
        self.execute_command(command);
    }
    pub fn execute_command(&self, command: Command){
        match command {
            Command::ShowPopup(notification) => {
                println!("Creating popup for: {}", notification.summary);
                let order = self.state.borrow_mut().active_popups_nb();
                
                let notif_pop = NotificationPopup::new(notification.clone(), &self.app_gtk, order);
                let notification_id = notif_pop.notification_id;
                
                self.state.borrow_mut().add_active_popup(notif_pop.clone());
                
                notif_pop.show_popup();
                
                let duration = notification.expire_timeout as u64;
                let app = self.clone();
                
                timeout_add_local(Duration::from_millis(duration), move ||{
                    app.handle_message(Message::PopupExpired(notification_id));
                    glib::ControlFlow::Break
                });
                
                println!("Popup shown, will expire in {}ms", duration);
            
            }
            Command::HidePopup(notif_id) => {
                println!("Command to hide popup with ID: {}", notif_id);

                let mut state = self.state.borrow_mut();

                if let Some(index) = state.active_popups.iter().position(|p| p.notification_id == notif_id) {
                    let popup = state.active_popups.remove(index);
                    drop(state);  

                    popup.window.close();
                    println!("Closed window for notification ID: {}", notif_id);
                } else {
                    println!("Popup with ID {} not found", notif_id);
                }
                self.handle_message(Message::RedrawWindows);
                self.handle_message(Message::SendNextNotification);
            }
            Command::RedrawWindows => {
                println!("Redrawing all windows");
                self.state.borrow_mut().redraw_windows();
            }
            Command::ToggleDoNotDisturb => {
                println!("The command is toggling do not disturb");
                self.state.borrow_mut().toggle_dnd();
                self.handle_message(Message::SendNextNotification);

                // self.state.borrow_mut().redraw_windows();
            }
            _ => {
                println!("Other type of command");
            }
        }
    }
}
