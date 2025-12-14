use std::{cell::RefCell, collections::HashMap, ops::ControlFlow, rc::Rc};

use gtk4::{glib::object::Cast, prelude::{BoxExt, GtkWindowExt, WidgetExt}, subclass::window, ApplicationWindow, Label};

use crate::dbus::{self, NotificationData};

pub enum NotificationEvent{
    ShowNotificationCenter,
    CloseNotificationCenter,
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
    notifications: Rc<RefCell<HashMap<u32, dbus::NotificationData>>>, 
    window_methods: WindowMethods,
}
// TODO: add timestamp, app
impl NotificationCenter{
    pub fn new(receiver: glib::Receiver<NotificationEvent>, notification_receiver: glib::Receiver<NotificationData>, window: ApplicationWindow) -> Self {
        Self {
            window_methods: WindowMethods::new(window),
            receiver: receiver,
            notification_receiver: notification_receiver,
            notifications: Rc::new(RefCell::new(HashMap::new())),
        }
        
    }
    pub fn attach_receiver(self){
        let notifications = self.notifications.clone();
        let window1 = self.window_methods.window.clone();
        let window2 = self.window_methods.window.clone();

        self.receiver.attach(None, move |event| {
            if let NotificationEvent::ShowNotificationCenter = event {
                window1.show();
            }
            if let NotificationEvent::CloseNotificationCenter = event {
                window1.hide();
            }
            glib::ControlFlow::Continue
        });

        self.notification_receiver.attach(None, move |_notification| {
            let notification_ = _notification.clone();
            let notifications_ = notifications.clone();
            notifications.borrow_mut().insert(_notification.id, _notification);
            add_notification(notification_, notifications_, &window2);
            glib::ControlFlow::Continue
        });

    }
}
pub fn add_notification(notification: NotificationData, notifications: Rc<RefCell<HashMap<u32, dbus::NotificationData>>>, window: &ApplicationWindow){
    if let Some(container) = window.child(){
        if let Some(box_container) = container.downcast_ref::<gtk4::Box>() {
            let notification_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            let notification_container_clone = notification_container.clone();
            let box_container_clone = box_container.clone();
            notification_container.add_css_class("notification-container");
            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_gesture, _n_press, _x, _y| {
                notifications.borrow_mut().remove(&notification.id);
                box_container_clone.remove(&notification_container_clone);
                println!("Notification clicked!");

            });
            notification_container.add_controller(gesture);

            let body = notification.body.clone();
            let summary = notification.summary.clone();
            let label_body = Label::new(Some(&body));
            let label_summary = Label::new(Some(&summary));
            label_body.add_css_class("label-body");
            label_summary.add_css_class("label-summary");
            label_body.set_wrap_mode(pango::WrapMode::WordChar);
            label_body.set_wrap(true); 
            label_body.set_max_width_chars(40);
            label_body.set_xalign(0.0);
            label_body.set_yalign(0.0);
            label_summary.set_xalign(0.0);
            label_summary.set_yalign(0.0);
            notification_container.append(&label_body); 
            notification_container.append(&label_summary);
            box_container.append(&notification_container);
        }
    }
}
