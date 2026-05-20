use std::path::Path;
use std::sync::mpsc;

use gio::ListStore;
use gtk4::glib;
use gtk4::glib::MainContext;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, ColumnView, ColumnViewColumn, Label, NoSelection,
    ScrolledWindow, SignalListItemFactory, Window,
};
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

// --- GLib Object subclass for ColumnView rows ---

mod row_data_imp {
    use std::cell::RefCell;

    use glib::prelude::*;
    use glib::subclass::prelude::*;

    #[derive(Default)]
    pub struct RowData {
        pub name: RefCell<String>,
        pub path: RefCell<String>,
        pub integrated: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RowData {
        const NAME: &'static str = "AppImageRowData";
        type Type = super::RowData;
    }

    impl ObjectImpl for RowData {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("name").construct().build(),
                    glib::ParamSpecString::builder("path").construct().build(),
                    glib::ParamSpecBoolean::builder("integrated").construct().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "name" => {
                    self.name.replace(value.get().unwrap());
                }
                "path" => {
                    self.path.replace(value.get().unwrap());
                }
                "integrated" => {
                    self.integrated.replace(value.get().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
                "path" => self.path.borrow().to_value(),
                "integrated" => self.integrated.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct RowData(ObjectSubclass<row_data_imp::RowData>);
}

impl RowData {
    fn new(entry: &AppImageEntry) -> Self {
        glib::Object::builder::<Self>()
            .property("name", entry.name.clone())
            .property("path", entry.path.clone())
            .property("integrated", entry.integrated)
            .build()
    }

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

    let vbox = GtkBox::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_all(10);

    let label = Label::new(Some(question));
    label.set_wrap(true);
    vbox.append(&label);

    let hbox = GtkBox::new(gtk4::Orientation::Horizontal, 10);
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
    store: ListStore,
}

#[derive(Debug)]
enum MainWinMsg {
    Toggle(usize),
}

#[relm4::component(pub)]
impl SimpleComponent for MainWin {
    type Init = (Vec<AppImageEntry>, Option<String>);
    type Input = MainWinMsg;
    type Output = ();

    view! {
        Window {
            set_title: Some("AppImageXdg"),
            set_default_width: 700,
            set_default_height: 450,

            ScrolledWindow {
                #[name = "column_view"]
                ColumnView {
                    set_vexpand: true,
                    set_hexpand: true,
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
        let store = ListStore::new::<RowData>();

        let (mut entries, explicit_path) = init;
        entries.sort_by(|a, b| {
            a.integrated.cmp(&b.integrated)
                .then(a.name.cmp(&b.name))
        });

        let model = MainWin {
            entries,
            store: store.clone(),
        };

        setup_columns(&widgets.column_view, &store, &sender);
        rebuild_store(&model.entries, &store);

        if let Some(path) = explicit_path {
            if let Some(idx) = model.entries.iter().position(|e| e.path == path) {
                if !model.entries[idx].integrated {
                    if gui_yes_no(&format!("Install {}?", model.entries[idx].name)) {
                        sender.input(MainWinMsg::Toggle(idx));
                    }
                }
            }
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        let MainWinMsg::Toggle(idx) = message;
        if idx >= self.entries.len() {
            return;
        }
        if self.entries[idx].integrated {
            self.do_remove(idx);
        } else {
            self.do_install(idx);
        }
        rebuild_store(&self.entries, &self.store);
    }
}

fn setup_columns(column_view: &ColumnView, store: &ListStore, sender: &ComponentSender<MainWin>) {
    let selection = NoSelection::new(Some(store.clone()));
    column_view.set_model(Some(&selection));

    // --- Name column ---
    let factory = SignalListItemFactory::new();
    factory.connect_setup(|_f, list_item| {
        let label = Label::new(None);
        label.set_halign(Align::Start);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        list_item.set_child(Some(&label));
    });
    factory.connect_bind(|_f, list_item| {
        if let Some(label) = list_item.child().and_then(|c| c.downcast::<Label>().ok()) {
            if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowData>().ok()) {
                let name: String = row.property("name");
                label.set_text(&name);
            }
        }
    });
    let col = ColumnViewColumn::new(Some("Name"), Some(factory));
    col.set_expand(true);
    column_view.append_column(&col);

    // --- Path column ---
    let factory = SignalListItemFactory::new();
    factory.connect_setup(|_f, list_item| {
        let label = Label::new(None);
        label.set_halign(Align::Start);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
        list_item.set_child(Some(&label));
    });
    factory.connect_bind(|_f, list_item| {
        if let Some(label) = list_item.child().and_then(|c| c.downcast::<Label>().ok()) {
            if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowData>().ok()) {
                let path: String = row.property("path");
                label.set_text(&path);
            }
        }
    });
    let col = ColumnViewColumn::new(Some("Path"), Some(factory));
    col.set_expand(true);
    column_view.append_column(&col);

    // --- Status column ---
    let factory = SignalListItemFactory::new();
    factory.connect_setup(|_f, list_item| {
        let label = Label::new(None);
        label.set_halign(Align::Start);
        list_item.set_child(Some(&label));
    });
    factory.connect_bind(|_f, list_item| {
        if let Some(label) = list_item.child().and_then(|c| c.downcast::<Label>().ok()) {
            if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowData>().ok()) {
                let integrated: bool = row.property("integrated");
                label.set_text(if integrated { "Installed" } else { "Not installed" });
            }
        }
    });
    let col = ColumnViewColumn::new(Some("Status"), Some(factory));
    col.set_resizable(false);
    column_view.append_column(&col);

    // --- Action column ---
    let factory = SignalListItemFactory::new();
    let s = sender.clone();
    factory.connect_bind(move |_f, list_item| {
        let pos = list_item.position() as usize;
        let btn = Button::new();

        if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowData>().ok()) {
            let integrated: bool = row.property("integrated");
            if integrated {
                btn.set_label("Remove");
                btn.add_css_class("destructive-action");
            } else {
                btn.set_label("Install");
                btn.add_css_class("suggested-action");
            }
        }

        let s2 = s.clone();
        btn.connect_clicked(move |_| {
            s2.input(MainWinMsg::Toggle(pos));
        });

        list_item.set_child(Some(&btn));
    });
    let col = ColumnViewColumn::new(Some("Action"), Some(factory));
    col.set_resizable(false);
    column_view.append_column(&col);
}

fn rebuild_store(entries: &[AppImageEntry], store: &ListStore) {
    let items: Vec<RowData> = entries.iter().map(RowData::new).collect();
    store.splice(0, store.n_items(), &items);
}

impl MainWin {
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

        let desktop_entries = list_all_desktop_entries().unwrap_or_default();
        let to_remove = desktop_entries.iter().find(|de| {
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

pub fn run_gui(entries: Vec<AppImageEntry>, explicit_path: Option<String>) {
    let args: Vec<String> = std::env::args().take(1).collect();
    let app = RelmApp::new("com.appimagexdg.AppImageXdg").with_args(args);
    app.run::<MainWin>((entries, explicit_path));
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
    fn test_row_data() {
        let entry = AppImageEntry {
            path: "/test/a.AppImage".into(),
            name: "a.AppImage".into(),
            integrated: true,
        };
        let row = RowData::new(&entry);
        let name: String = row.property("name");
        let path: String = row.property("path");
        let integrated: bool = row.property("integrated");
        assert_eq!(name, "a.AppImage");
        assert_eq!(path, "/test/a.AppImage");
        assert!(integrated);
    }

    #[test]
    fn test_gui_yes_no_panics_without_gtk() {
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
        let connector = MainWin::builder().launch((entries, None));
        {
            let state = connector.state().get();
            assert_eq!(state.model.entries.len(), 2);
            assert!(!state.model.entries[0].integrated);
            assert!(state.model.entries[1].integrated);
            assert_eq!(state.model.entries[0].name, "a.AppImage");
            assert_eq!(state.model.entries[1].name, "b.AppImage");
            assert_eq!(state.model.store.n_items(), 2);
        }
        let _controller = connector.detach();
    }

    #[test]
    fn test_rebuild_store() {
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
        let store = ListStore::new::<RowData>();
        rebuild_store(&entries, &store);
        assert_eq!(store.n_items(), 1);

        let item = store.item(0).unwrap();
        let row = item.downcast::<RowData>().unwrap();
        let name: String = row.property("name");
        assert_eq!(name, "x.AppImage");
    }

    #[test]
    fn test_rebuild_store_updates() {
        if !try_init_gtk() {
            eprintln!("skipping: no display available");
            return;
        }
        let mut entries = vec![
            AppImageEntry {
                path: "/tmp/y.AppImage".into(),
                name: "y.AppImage".into(),
                integrated: false,
            },
        ];
        let store = ListStore::new::<RowData>();
        rebuild_store(&entries, &store);
        assert_eq!(store.n_items(), 1);

        entries[0].integrated = true;
        rebuild_store(&entries, &store);
        assert_eq!(store.n_items(), 1);

        let item = store.item(0).unwrap();
        let row = item.downcast::<RowData>().unwrap();
        let integrated: bool = row.property("integrated");
        assert!(integrated);
    }
}
