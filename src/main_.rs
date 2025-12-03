use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label};
use glib::timeout_add_local;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::rc::Rc;

fn main() {
    let app = Application::builder()
        .application_id("org.example.PopupDemo")
        .build();

    app.connect_activate(|app| {
        // Create a borderless popup window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Popup")
            .default_width(300)
            .default_height(100)
            .decorated(false)
            .build();

        let label = Label::new(Some("Popup"));
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_default_size(300, 100);
        window.set_opacity(0.95);
        window.set_child(Some(&label));
        let anchors = [
            (Edge::Left, false),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ];
        for (anchor, state) in anchors {
            window.set_anchor(anchor, state);
        }
        window.present();

        // Clone window for the closure
        let window = Rc::new(window);
        let win_clone = window.clone();

        // Hide the window after 3 seconds
        timeout_add_local(std::time::Duration::from_secs(3), move || {
            win_clone.hide();
            glib::ControlFlow::Continue
        });
    });

    app.run();
}

