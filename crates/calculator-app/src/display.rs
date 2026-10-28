use gtk::pango;
use gtk::prelude::*;

pub struct DisplayView {
    pub root: gtk::Box,
    pub tape_label: gtk::Label,
    pub preview_label: gtk::Label,
    pub readout_label: gtk::Label,
    pub mem_badge: gtk::Label,
    pub error_badge: gtk::Label,
}

impl DisplayView {
    pub fn new() -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
        root.add_css_class("calc-display");

        // Top line: tape and badges
        let top_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let mem_badge = gtk::Label::new(Some("M"));
        mem_badge.add_css_class("calc-badge");
        mem_badge.set_visible(false);
        top_row.append(&mem_badge);

        let tape_label = gtk::Label::new(None);
        tape_label.add_css_class("calc-tape");
        tape_label.set_hexpand(true);
        tape_label.set_xalign(1.0);
        tape_label.set_ellipsize(pango::EllipsizeMode::Start);
        top_row.append(&tape_label);
        root.append(&top_row);

        // Second line: live preview
        let preview_label = gtk::Label::new(None);
        preview_label.add_css_class("calc-preview");
        preview_label.set_xalign(1.0);
        preview_label.set_visible(false);
        root.append(&preview_label);

        // Third line: primary readout
        let readout_label = gtk::Label::new(Some("0"));
        readout_label.add_css_class("calc-readout");
        readout_label.set_xalign(1.0);
        readout_label.set_ellipsize(pango::EllipsizeMode::Start);
        readout_label.set_selectable(true);
        root.append(&readout_label);

        // Fourth line: error indicator badge (hidden when no error)
        let error_badge = gtk::Label::new(None);
        error_badge.add_css_class("calc-error-badge");
        error_badge.set_halign(gtk::Align::End);
        error_badge.set_visible(false);
        root.append(&error_badge);

        Self { root, tape_label, preview_label, readout_label, mem_badge, error_badge }
    }

    pub fn set_tape(&self, text: &str) {
        self.tape_label.set_text(text);
    }

    pub fn set_preview(&self, text: Option<&str>) {
        if let Some(t) = text {
            self.preview_label.set_text(&format!("= {t}"));
            self.preview_label.set_visible(true);
        } else {
            self.preview_label.set_text("");
            self.preview_label.set_visible(false);
        }
    }

    pub fn set_readout(&self, text: &str) {
        self.readout_label.set_text(text);
        // Dynamically scale font size if the number gets very long
        if text.len() > 14 {
            self.readout_label.remove_css_class("calc-readout");
            self.readout_label.add_css_class("calc-readout-small");
        } else {
            self.readout_label.remove_css_class("calc-readout-small");
            self.readout_label.add_css_class("calc-readout");
        }
    }

    pub fn set_memory_active(&self, active: bool) {
        self.mem_badge.set_visible(active);
    }

    pub fn set_error(&self, error: Option<&str>) {
        if let Some(msg) = error {
            self.error_badge.set_text(msg);
            self.error_badge.set_visible(true);
        } else {
            self.error_badge.set_visible(false);
        }
    }
}
