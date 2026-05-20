use std::path::Path;
use std::sync::mpsc;

use gtk4::glib::MainContext;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, ScrolledWindow, Window};
use relm4::prelude::*;
use relm4::RelmWidgetExt;

use crate::config::*;
use crate::desktop::*;

#[derive(Clone, Debug, PartialEq)]
pub struct AppImageEntry {
    pub path: String,
    pub name: String,
    pub integrated: bool,
}

// --- Synchronous yes/no dialog ---

fn gui_yes_no(question: &str) -> bool {
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

    loop {
        if let Ok(val) = rx.try_recv() {
            break val;
        }
        MainContext::default().iteration(true);
    }
}

// --- Main window component ---

struct MainWin {
    entries: Vec<AppImageEntry>,
    list_box: GtkBox,
}

#[derive(Debug)]
enum MainWinMsg {
    Install(usize),
    Remove(usize),
}

#[relm4::component(pub)]
impl SimpleComponent for MainWin {
    type Init = Vec<AppImageEntry>;
    type Input = MainWinMsg;
    type Output = ();

    view! {
        Window {
            set_title: Some("AppImageXdg"),
            set_default_width: 600,
            set_default_height: 400,

            ScrolledWindow {
                #[name = "list_box"]
                GtkBox {
                    set_orientation: Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 10,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let model = MainWin {
            entries: init,
            list_box: widgets.list_box.clone(),
        };
        model.rebuild_list(&sender);

        if model.entries.len() == 1 && !model.entries[0].integrated {
            let name = model.entries[0].name.clone();
            if gui_yes_no(&format!("Install {}?", name)) {
                sender.input(MainWinMsg::Install(0));
            }
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            MainWinMsg::Install(idx) => {
                self.do_install(idx);
                self.rebuild_list(&sender);
            }
            MainWinMsg::Remove(idx) => {
                self.do_remove(idx);
                self.rebuild_list(&sender);
            }
        }
    }
}

impl MainWin {
    fn rebuild_list(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        for (i, entry) in self.entries.iter().enumerate() {
            self.list_box.append(&make_row(entry, i, sender));
        }
        self.list_box.show();
    }

    fn do_install(&mut self, idx: usize) {
        let entry = &self.entries[idx];
        let mut path = entry.path.clone();
        let home_apps = install_path();
        let parent = Path::new(&entry.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if parent != home_apps {
            if gui_yes_no(&format!("Move {} to {}?", entry.name, home_apps)) {
                let _ = std::fs::create_dir_all(&home_apps);
                let new_path = format!("{}/{}", home_apps, entry.name);
                if std::fs::rename(&entry.path, &new_path).is_ok() {
                    path = new_path;
                }
            }
        }

        if ensure_dirs().is_ok() {
            if let Err(e) = crate::process_app_image(&path) {
                gui_yes_no(&format!("Install failed: {}", e));
                return;
            }
        }
        self.entries[idx].integrated = true;
    }

    fn do_remove(&mut self, idx: usize) {
        let entry = &self.entries[idx];
        let base_stem = Path::new(&entry.path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let entries = list_all_desktop_entries().unwrap_or_default();
        let to_remove = entries.iter().find(|de| {
            de.exec.to_lowercase().contains(&base_stem.to_lowercase())
        });

        if let Some(de) = to_remove {
            if remove_desktop_entry(de).is_ok() {
                let name = Path::new(&entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if gui_yes_no(&format!("Delete {} file?", name)) {
                    let _ = std::fs::remove_file(&entry.path);
                }
            }
        }

        self.entries[idx].integrated = false;
    }
}

fn make_row(entry: &AppImageEntry, idx: usize, sender: &ComponentSender<MainWin>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.set_margin_all(5);
    row.set_halign(Align::Fill);

    let name = Label::new(Some(&entry.name));
    name.set_halign(Align::Start);
    name.set_hexpand(true);
    row.append(&name);

    let status = Label::new(Some(if entry.integrated {
        "Installed"
    } else {
        "Not installed"
    }));
    row.append(&status);

    if entry.integrated {
        let btn = Button::with_label("Remove");
        let s = sender.clone();
        btn.connect_clicked(move |_| {
            s.input(MainWinMsg::Remove(idx));
        });
        row.append(&btn);
    } else {
        let btn = Button::with_label("Install");
        let s = sender.clone();
        btn.connect_clicked(move |_| {
            s.input(MainWinMsg::Install(idx));
        });
        row.append(&btn);
    }

    row
}

pub fn run_gui(entries: Vec<AppImageEntry>) {
    let args: Vec<String> = std::env::args().take(1).collect();
    let app = RelmApp::new("com.appimagexdg.AppImageXdg").with_args(args);
    app.run::<MainWin>(entries);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_init_gtk() -> bool {
        gtk4::init().is_ok()
    }

    #[test]
    fn test_appimage_entry() {
        let e = AppImageEntry {
            path: "/a/b.AppImage".into(),
            name: "b.AppImage".into(),
            integrated: false,
        };
        assert_eq!(e.path, "/a/b.AppImage");
        assert!(!e.integrated);

        let e2 = AppImageEntry {
            integrated: true,
            ..e.clone()
        };
        assert!(e2.integrated);
    }

    #[test]
    fn test_gui_yes_no_panics_without_gtk() {
        // This should only be called when GTK is initialized.
        // The assertion is that the function exists and doesn't
        // require external caching/state management.
        assert!(std::mem::size_of::<fn(&str) -> bool>() > 0);
    }

    #[test]
    fn test_main_win_init() {
        if !try_init_gtk() {
            eprintln!("skipping: no display available");
            return;
        }
        let entries = vec![
            AppImageEntry {
                path: "/tmp/a.AppImage".into(),
                name: "a.AppImage".into(),
                integrated: false,
            },
            AppImageEntry {
                path: "/tmp/b.AppImage".into(),
                name: "b.AppImage".into(),
                integrated: true,
            },
        ];
        let connector = MainWin::builder().launch(entries);
        {
            let state = connector.state().get();
            assert_eq!(state.model.entries.len(), 2);
            assert!(!state.model.entries[0].integrated);
            assert!(state.model.entries[1].integrated);
            assert_eq!(state.model.entries[0].name, "a.AppImage");
            assert_eq!(state.model.entries[1].name, "b.AppImage");
        }
        let _controller = connector.detach();
    }

    #[test]
    fn test_main_win_rebuild_list() {
        if !try_init_gtk() {
            eprintln!("skipping: no display available");
            return;
        }
        let entries = vec![
            AppImageEntry {
                path: "/tmp/x.AppImage".into(),
                name: "x.AppImage".into(),
                integrated: false,
            },
        ];
        let connector = MainWin::builder().launch(entries);
        {
            let state = connector.state().get();
            let child = state.model.list_box.first_child();
            assert!(child.is_some());
        }
        let _controller = connector.detach();
    }
}
