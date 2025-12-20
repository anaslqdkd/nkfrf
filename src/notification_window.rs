use gtk4::{Application, ApplicationWindow, Box, Label, prelude::{BoxExt, GtkWindowExt, WidgetExt}};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

const TOP_MARGIN: i32 = 20;
const WINDOW_HEIGHT: i32 = 70;
const WINDOW_WIDTH: i32 = 270;

pub struct NotificationWindow{
    summary: String,
    body: String,
}
impl NotificationWindow {
    pub fn new(summary: &str, body: &str) -> Self{
        Self {
            summary: summary.to_string(),
            body: body.to_string(),
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
        println!("The notification in add content is: {}, {}",self.summary, self.body);

        let label_body = Label::new(Some(&self.body));
        let label_summary = Label::new(Some(&self.summary));
        label_body.add_css_class("label-body");
        label_body.set_wrap(true);
        label_body.set_wrap_mode(pango::WrapMode::WordChar);
        label_body.set_max_width_chars(40);
        label_body.set_xalign(0.0);
        label_body.set_yalign(0.0);
        label_summary.set_xalign(0.0);
        label_summary.set_yalign(0.0);
        label_summary.add_css_class("label-summary");
        container.append(&label_summary);
        container.append(&label_body);
        container
    }
    fn create_window(&self, app: &Application) -> ApplicationWindow{
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(WINDOW_WIDTH)
            .default_height(WINDOW_HEIGHT)
            .build();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_opacity(0.95);
        // FIXME: add the stack number if multiple windows
        let stack_number = 0;
        let y_offset = stack_number * WINDOW_HEIGHT + TOP_MARGIN;
        window.set_margin(Edge::Right, TOP_MARGIN);
        window.set_margin(Edge::Top, y_offset);
        
        window
    }

    pub fn build(self, app: &Application) -> ApplicationWindow{
        let window = self.create_window(app);
        let content = self.add_content();
        Self::set_anchors(&window);
        window.set_child(Some(&content));
        window
    }

}
