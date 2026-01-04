//! Basic example showing how to integrate gtk-layout-inspector.
//!
//! Run with: cargo run --example basic --features server

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use glib::ControlFlow;
use gtk4::prelude::*;
use gtk4::{self as gtk, Application, ApplicationWindow, Box, Button, Entry, Label, Orientation};

use gtk_layout_inspector::server::{self, Command};
use gtk_layout_inspector::{dump_widget_tree, find_button_by_label, find_entry_by_placeholder};

fn main() {
    let app = Application::builder()
        .application_id("com.example.gtk-inspector-demo")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Build UI
    let window = ApplicationWindow::builder()
        .application(app)
        .title("GTK Inspector Demo")
        .default_width(400)
        .default_height(300)
        .build();

    let vbox = Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    let label = Label::new(Some("Enter your name:"));
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Name"));

    let button = Button::with_label("Greet");
    let result_label = Label::new(None);

    // Connect button click
    let entry_clone = entry.clone();
    let result_clone = result_label.clone();
    button.connect_clicked(move |_| {
        let name = entry_clone.text();
        result_clone.set_text(&format!("Hello, {}!", name));
    });

    vbox.append(&label);
    vbox.append(&entry);
    vbox.append(&button);
    vbox.append(&result_label);

    window.set_child(Some(&vbox));
    window.present();

    // Initialize the debug server
    let (mut cmd_rx, _guard) = server::init();
    println!("Debug server listening on: {}", server::socket_path().display());

    // Keep the guard alive by storing it
    let guard = Rc::new(RefCell::new(Some(_guard)));

    // Store window reference for the closure
    let window_weak = window.downgrade();
    let guard_clone = guard.clone();

    // Poll for commands in GTK main loop
    glib::timeout_add_local(Duration::from_millis(50), move || {
        // Keep guard alive
        let _guard = guard_clone.borrow();

        while let Ok(cmd) = cmd_rx.try_recv() {
            let Some(window) = window_weak.upgrade() else {
                continue;
            };

            match cmd {
                Command::Dump { respond } => {
                    let dump = dump_widget_tree(&window);
                    let _ = respond.send(dump.to_string());
                }
                Command::DumpJson { respond } => {
                    let dump = dump_widget_tree(&window);
                    let _ = respond.send(dump.to_json());
                }
                Command::Click { label, respond } => {
                    if let Some(button) = find_button_by_label(&window, &label) {
                        button.emit_clicked();
                        let _ = respond.send(Ok(()));
                    } else {
                        let _ = respond.send(Err(format!("Button '{}' not found", label)));
                    }
                }
                Command::Input { field, value, respond } => {
                    if let Some(entry) = find_entry_by_placeholder(&window, &field) {
                        entry.set_text(&value);
                        let _ = respond.send(Ok(()));
                    } else {
                        let _ = respond.send(Err(format!("Entry '{}' not found", field)));
                    }
                }
                Command::Submit { respond } => {
                    // Activate the focused widget
                    if let Some(focus) = window.focus() {
                        focus.activate();
                        let _ = respond.send(Ok(()));
                    } else {
                        let _ = respond.send(Err("No focused widget".to_string()));
                    }
                }
            }
        }
        ControlFlow::Continue
    });
}
