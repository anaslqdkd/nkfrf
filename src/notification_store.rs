use std::{cell::RefCell, ops::ControlFlow, rc::Rc};

use gtk4::{prelude::{BoxExt, GtkWindowExt, WidgetExt}, subclass::window, ApplicationWindow, Label};

use crate::dbus::{self, NotificationData};

pub enum NotificationEvent{
    ShowNotificationCenter,
}
#[derive(Debug, Clone)]
pub struct WindowMethods{
    window: ApplicationWindow,
}
impl WindowMethods{
    pub fn new(window: ApplicationWindow) -> Self{
        Self{
            window: window,
        }
    }
    pub fn show_window(&mut self){
        self.window.show();
    }
}

#[derive(Debug)]
pub struct NotificationCenter{
    receiver: glib::Receiver<NotificationEvent>,
    notification_receiver: glib::Receiver<NotificationData>,
    notifications: Vec<dbus::NotificationData>, 
    window_methods: WindowMethods,
}
impl NotificationCenter{
    pub fn new(receiver: glib::Receiver<NotificationEvent>, notification_receiver: glib::Receiver<NotificationData>, window: ApplicationWindow) -> Self {
        Self {
            window_methods: WindowMethods::new(window),
            receiver: receiver,
            notification_receiver: notification_receiver,
            notifications: Vec::new(),
        }
        
    }
    pub fn attach_receiver(self){
        let mut notifications = self.notifications.clone();
        let window1 = self.window_methods.window.clone();
        let window2 = self.window_methods.window.clone();

        self.receiver.attach(None, move |event| {
            if let NotificationEvent::ShowNotificationCenter = event {
                window1.show();
            }
            glib::ControlFlow::Continue
        });

        self.notification_receiver.attach(None, move |_notification| {
            notifications.push(_notification);
            show_notifications(&notifications, &window2);
            glib::ControlFlow::Continue
        });

}
}
pub fn show_notifications(notifications: &Vec<NotificationData>, window: &ApplicationWindow){
    // FIXME: change the css class
    let window = window;
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    container.add_css_class("container-bg");
    window.set_child(Some(&container));
    for notification in notifications.iter(){
        let body = notification.body.clone();
        let summary = notification.summary.clone();
        let label_body = Label::new(Some(&body));
        let label_summary = Label::new(Some(&summary));
        label_body.add_css_class("label-body");
        label_body.set_wrap_mode(pango::WrapMode::WordChar);
        label_body.set_wrap(true); 
        label_body.set_max_width_chars(40);
        label_body.set_xalign(0.0);
        label_body.set_yalign(0.0);
        label_summary.set_xalign(0.0);
        label_summary.set_yalign(0.0);
        // label.add_css_class("notification-item"); 
        container.append(&label_body); 
        container.append(&label_summary);
    }
}
