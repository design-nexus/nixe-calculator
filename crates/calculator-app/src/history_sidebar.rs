use calculator_storage::{CalculatorDb, HistoryEntry};
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct HistorySidebar {
    pub root: gtk::Box,
    list_box: gtk::Box,
    search_entry: gtk::SearchEntry,
    db: CalculatorDb,
    on_restore: Rc<dyn Fn(String)>,
    on_toast: Rc<dyn Fn(&str)>,
}

impl HistorySidebar {
    pub fn new(db: CalculatorDb, on_restore: impl Fn(String) + 'static, on_toast: impl Fn(&str) + 'static) -> Rc<RefCell<Self>> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("calc-history-sidebar");

        let on_restore = Rc::new(on_restore);
        let on_toast = Rc::new(on_toast);

        // Header row
        let head = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        head.add_css_class("calc-history-head");

        let title = gtk::Label::new(Some("HISTORY"));
        title.add_css_class("calc-history-title");
        title.set_hexpand(true);
        title.set_xalign(0.0);
        head.append(&title);

        let clear_btn =
            gtk::Button::builder().icon_name("user-trash-symbolic").tooltip_text("Clear all history").has_frame(false).build();
        clear_btn.add_css_class("history-action-btn");
        head.append(&clear_btn);
        root.append(&head);

        // Search entry
        let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        search_box.set_margin_start(8);
        search_box.set_margin_end(8);
        search_box.set_margin_top(6);
        search_box.set_margin_bottom(6);
        let search_entry = gtk::SearchEntry::new();
        search_entry.set_hexpand(true);
        search_entry.set_placeholder_text(Some("Search calculations..."));
        search_box.append(&search_entry);
        root.append(&search_box);

        // List inside ScrolledWindow
        let list_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::External)
            .vexpand(true)
            .child(&list_box)
            .build();
        root.append(&scroll);

        let sidebar = Rc::new(RefCell::new(Self {
            root,
            list_box,
            search_entry: search_entry.clone(),
            db: db.clone(),
            on_restore,
            on_toast: on_toast.clone(),
        }));

        // Connect search change
        let weak = Rc::downgrade(&sidebar);
        search_entry.connect_search_changed(move |_| {
            if let Some(s) = weak.upgrade() {
                s.borrow().refresh();
            }
        });

        // Connect clear all
        let weak_clear = Rc::downgrade(&sidebar);
        clear_btn.connect_clicked(move |_| {
            if let Some(s) = weak_clear.upgrade() {
                let _ = s.borrow().db.clear_history();
                s.borrow().refresh();
                (s.borrow().on_toast)("History cleared");
            }
        });

        sidebar.borrow().refresh();
        sidebar
    }

    pub fn refresh(&self) {
        // Clear children
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let query = self.search_entry.text().to_string();
        let query_opt = if query.trim().is_empty() { None } else { Some(query.as_str()) };

        let entries = match self.db.list_history(query_opt, 50, 0) {
            Ok(e) => e,
            Err(_) => return,
        };

        if entries.is_empty() {
            let empty = gtk::Label::new(Some("No calculations yet"));
            empty.add_css_class("dim");
            empty.set_margin_top(30);
            self.list_box.append(&empty);
            return;
        }

        for entry in entries {
            let card = self.create_card(&entry);
            self.list_box.append(&card);
        }
    }

    fn create_card(&self, entry: &HistoryEntry) -> gtk::Box {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 2);
        card.add_css_class("calc-history-card");
        if entry.is_pinned {
            card.add_css_class("pinned");
        }

        // Top line: expression and actions
        let row1 = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let expr_lbl = gtk::Label::new(Some(&entry.expression));
        expr_lbl.add_css_class("calc-history-expr");
        expr_lbl.set_hexpand(true);
        expr_lbl.set_xalign(0.0);
        row1.append(&expr_lbl);

        // Pin button
        let pin_icon = if entry.is_pinned { "view-pin-symbolic" } else { "bookmark-new-symbolic" };
        let pin_btn = gtk::Button::builder()
            .icon_name(pin_icon)
            .tooltip_text(if entry.is_pinned { "Unpin" } else { "Pin to top" })
            .has_frame(false)
            .build();
        pin_btn.add_css_class("history-action-btn");
        let db = self.db.clone();
        let id = entry.id;
        let weak_card = card.downgrade();
        pin_btn.connect_clicked(move |b| {
            if let Ok(pinned) = db.toggle_pin(id) {
                b.set_icon_name(if pinned { "view-pin-symbolic" } else { "bookmark-new-symbolic" });
                b.set_tooltip_text(Some(if pinned { "Unpin" } else { "Pin to top" }));
                if let Some(c) = weak_card.upgrade() {
                    if pinned {
                        c.add_css_class("pinned");
                    } else {
                        c.remove_css_class("pinned");
                    }
                }
            }
        });
        row1.append(&pin_btn);

        // Delete button
        let del_btn =
            gtk::Button::builder().icon_name("edit-delete-symbolic").tooltip_text("Delete calculation").has_frame(false).build();
        del_btn.add_css_class("history-action-btn");
        let db_del = self.db.clone();
        let list_box = self.list_box.clone();
        let card_to_remove = card.clone();
        del_btn.connect_clicked(move |_| {
            let _ = db_del.delete_history(id);
            list_box.remove(&card_to_remove);
        });
        row1.append(&del_btn);

        card.append(&row1);

        // Bottom line: result
        let res_lbl = gtk::Label::new(Some(&format!("= {}", entry.result)));
        res_lbl.add_css_class("calc-history-res");
        res_lbl.set_xalign(0.0);
        card.append(&res_lbl);

        // Click to restore formula
        let gesture = gtk::GestureClick::new();
        let on_restore = self.on_restore.clone();
        let formula = entry.raw_result.clone();
        gesture.connect_pressed(move |_, _, _, _| {
            on_restore(formula.clone());
        });
        card.add_controller(gesture);

        card
    }
}
