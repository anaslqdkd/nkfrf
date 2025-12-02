use std::fs;

use gtk4::gdk::Display;
use gtk4::gio::DBusConnection;
use gtk4::{self as gtk, Button, CheckButton, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use serde_json::Value;
use zbus::{Connection, fdo};
use zbus::zvariant::OwnedValue;

fn activate(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    let provider = CssProvider::new();

    provider.load_from_path("style.css");

    let display = Display::default().expect("Could not connect to display");
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.init_layer_shell();

    window.set_layer(Layer::Overlay);
    window.set_default_size(200, 600);
    window.set_opacity(0.95);


    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.add_css_class("container-bg");

    let button = Button::with_label("Idk");
    button.add_css_class("my-button");
    button.remove_css_class("text-button");

    container.append(&button);

    window.set_margin(Edge::Right, 40);
    window.set_margin(Edge::Top, 20);

    let content = fs::read_to_string("/home/ash/org.json").expect("The file was not found");
    let json:Value = serde_json::from_str(&content).expect("bu");
    for item in json.as_array().expect("Expected json as an array"){
        let text = item.as_str().expect("Expected a string");
        let cb = CheckButton::with_label(text);
        cb.add_css_class("my-checkbox");
        container.append(&cb);
    }

    // ... or like this
    // Anchors are if the window is pinned to each edge of the output
    let anchors = [
        (Edge::Left, false),
        (Edge::Right, true),
        (Edge::Top, true),
        (Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        window.set_anchor(anchor, state);
    }
    window.set_child(Some(&container));

    window.show()
}

// fn main() {
//     let application = gtk::Application::new(Some("sh.wmww.gtk-layer-example"), Default::default());
//
//     application.connect_activate(|app| {
//         activate(app);
//     });
//     application.run();
//
// }


