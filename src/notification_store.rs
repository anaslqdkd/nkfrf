use std::{cell::RefCell, collections::HashMap, ops::ControlFlow, rc::Rc};

use gtk4::{
    // ApplicationWindow, Label,
    ApplicationWindow,
    Label,
    glib::object::Cast,
    prelude::{BoxExt, GtkWindowExt, WidgetExt},
    subclass::window,
};

use crate::dbus::{self, NotificationData};

pub enum NotificationEvent {
    ShowNotificationCenter,
    CloseNotificationCenter,
}
#[derive(Debug, Clone)]
pub struct WindowMethods {
    window: ApplicationWindow,
}
impl WindowMethods {
    pub fn new(window: ApplicationWindow) -> Self {
        Self { window }
    }
    pub fn show_window(&mut self) {
        self.window.show();
    }
}

#[derive(Debug)]
pub struct NotificationCenter {
    receiver: glib::Receiver<NotificationEvent>,
    notification_receiver: glib::Receiver<NotificationData>,
    notifications: Rc<RefCell<HashMap<u32, dbus::NotificationData>>>,
    window_methods: WindowMethods,
}
// TODO: add timestamp, app
impl NotificationCenter {
    pub fn new(
        receiver: glib::Receiver<NotificationEvent>,
        notification_receiver: glib::Receiver<NotificationData>,
        window: ApplicationWindow,
    ) -> Self {
        Self {
            window_methods: WindowMethods::new(window),
            receiver,
            notification_receiver,
            notifications: Rc::new(RefCell::new(HashMap::new())),
        }
    }
    pub fn attach_receiver(self) {
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

        self.notification_receiver
            .attach(None, move |_notification| {
                let notification_ = _notification.clone();
                let notifications_ = notifications.clone();
                notifications
                    .borrow_mut()
                    .insert(_notification.id, _notification);
                add_notification(notification_, notifications_, &window2);
                glib::ControlFlow::Continue
            });
    }
}
pub fn add_notification(
    notification: NotificationData,
    notifications: Rc<RefCell<HashMap<u32, dbus::NotificationData>>>,
    window: &ApplicationWindow,
) {
    let Some(container) = window.child() else {
        return;
    };
    let Some(box_container) = container.downcast_ref::<gtk4::Box>() else {
        return;
    };

    let notification_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    notification_container.add_css_class("notification-container");

    let gesture = gtk4::GestureClick::new();
    let notification_id = notification.id;
    let container_ref = notification_container.clone();
    let box_ref = box_container.clone();
    gesture.connect_pressed(move |_gesture, _n_press, _x, _y| {
        notifications.borrow_mut().remove(&notification_id);
        box_ref.remove(&container_ref);
        println!("Notification clicked!");
    });

    let label_summary = Label::new(Some(&notification.summary));
    label_summary.add_css_class("label-summary");
    label_summary.set_xalign(0.0);
    label_summary.set_yalign(0.0);

    let label_body = Label::new(Some(&notification.body));
    label_body.add_css_class("label-body");
    label_body.set_wrap_mode(pango::WrapMode::WordChar);
    label_body.set_wrap(true);
    label_body.set_max_width_chars(40);
    label_body.set_xalign(0.0);
    label_body.set_yalign(0.0);

    notification_container.append(&label_summary);
    notification_container.append(&label_body);
    box_container.append(&notification_container);
}
