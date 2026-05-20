use std::sync::mpsc;
use std::sync::OnceLock;

use gtk4::glib::MainContext;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Window};
use relm4::RelmWidgetExt;

static GTK_INIT: OnceLock<bool> = OnceLock::new();

pub fn gui_yes_no(question: &str) -> Option<bool> {
    let ok = *GTK_INIT.get_or_init(|| gtk4::init().is_ok());
    if !ok {
        return None;
    }

    let (tx, rx) = mpsc::channel();

    let window = Window::builder()
        .title("AppImageXdg")
        .default_width(350)
        .default_height(150)
        .modal(true)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 10);
    vbox.set_margin_all(10);

    let label = Label::new(Some(question));
    label.set_wrap(true);
    vbox.append(&label);

    let hbox = GtkBox::new(Orientation::Horizontal, 10);
    hbox.set_halign(Align::End);

    let no = Button::with_label("No");
    let yes = Button::with_label("Yes");

    let chan = tx.clone();
    let w = window.clone();
    no.connect_clicked(move |_| {
        chan.send(false).ok();
        w.close();
    });

    let w = window.clone();
    yes.connect_clicked(move |_| {
        tx.send(true).ok();
        w.close();
    });

    hbox.append(&no);
    hbox.append(&yes);
    vbox.append(&hbox);
    window.set_child(Some(&vbox));
    window.present();

    let result = loop {
        if let Ok(val) = rx.try_recv() {
            break val;
        }
        MainContext::default().iteration(true);
    };

    Some(result)
}
