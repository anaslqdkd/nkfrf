use gtk4::{Application, ApplicationWindow, Box, Label, prelude::{BoxExt, GtkWindowExt, WidgetExt}};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::dbus::Notification;

const TOP_MARGIN: i32 = 20;
const WINDOW_HEIGHT: i32 = 70;
const WINDOW_WIDTH: i32 = 270;

pub struct NotificationWindow{
    summary: String,
    body: String,
    urgency: u8,
}
impl NotificationWindow {
    pub fn new(notification: Notification) -> Self{
        let summary = notification.summary;
        let body = notification.body;
        Self {
            summary: summary.to_string(),
            body: body.to_string(),
            urgency: notification.hints.urgency,
        }
    }
    fn set_anchors(window: &ApplicationWindow){
        let anchors = [
            (Edge::Left, false),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ];
        for (anchor, state) in anchors {
            window.set_anchor(anchor, state);
        }
    }
    fn add_content(&self) -> Box {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.add_css_class("container-bg");
        
        // Add urgency-specific CSS class
        let urgency_class = match self.urgency {
            0 => "urgency-low",
            1 => "urgency-normal",
            2 => "urgency-critical",
            _ => "urgency-normal", // Default to normal for unknown values
        };
        container.add_css_class(urgency_class);
        
        println!("The notification in add content is: {}, {} (urgency: {})",
                 self.summary, self.body, self.urgency);

        let label_body = Label::new(Some(&self.body));
        let label_summary = Label::new(Some(&self.summary));
        label_body.add_css_class("label-body");
        label_body.set_wrap(true);
        label_body.set_wrap_mode(pango::WrapMode::WordChar);
        label_body.set_max_width_chars(40);
        label_body.set_lines(2);
        label_body.set_ellipsize(pango::EllipsizeMode::End);
        label_body.set_xalign(0.0);
        label_body.set_yalign(0.0);
        label_summary.set_xalign(0.0);
        label_summary.set_yalign(0.0);
        label_summary.set_ellipsize(pango::EllipsizeMode::End);
        label_summary.add_css_class("label-summary");
        container.append(&label_summary);
        container.append(&label_body);
        container
    }
    fn create_window(&self, app: &Application, order_nb: u32) -> ApplicationWindow{
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(WINDOW_WIDTH)
            .default_height(WINDOW_HEIGHT)
            .build();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_opacity(0.95);
        let stack_number = order_nb as i32;
        // TODO: manage this for max_lines != 2
        let y_offset = stack_number * WINDOW_HEIGHT + TOP_MARGIN;
        window.set_margin(Edge::Right, TOP_MARGIN);
        window.set_margin(Edge::Top, y_offset);
        
        window
    }

    pub fn build(self, app: &Application, order_nb: u32) -> ApplicationWindow{
        let window = self.create_window(app, order_nb);
        let content = self.add_content();
        Self::set_anchors(&window);
        window.set_child(Some(&content));
        window
    }

}

