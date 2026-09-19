use gtk4::{self as gtk, prelude::*};

/// Anything that contributes a widget to the bar.
///
/// A module owns its widgets and keeps them up to date on its own, driven by
/// the glib main loop (a timer, a watched fd, a GTK signal). The bar therefore
/// only has to ask each module for its root widget once, at startup.
pub trait Module {
    fn widget(&self) -> gtk::Widget;
}

/// Where along the bar a module is placed.
#[derive(Clone, Copy)]
pub enum Section {
    Left,
    /// The expanding slot: it soaks up whatever width the other two leave.
    Center,
    Right,
}

/// A plain label with the bar's module styling, for modules that are just text.
pub fn label(class: &str) -> gtk::Label {
    let label = gtk::Label::new(None);
    label.add_css_class("module");
    label.add_css_class(class);
    label
}

/// The bar's root widget plus its three slots.
pub struct Bar {
    root: gtk::CenterBox,
    left: gtk::Box,
    center: gtk::Box,
    right: gtk::Box,
}

impl Bar {
    pub fn new() -> Self {
        let slot = |class: &str| {
            let slot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            slot.add_css_class(class);
            slot
        };
        let left = slot("left");
        let center = slot("center");
        let right = slot("right");

        let root = gtk::CenterBox::new();
        root.add_css_class("bar");

        root.set_start_widget(Some(&left));
        root.set_center_widget(Some(&center));
        root.set_end_widget(Some(&right));

        Self { root, left, center, right }
    }

    /// Append `module`'s widget to `section`.
    pub fn push(&self, section: Section, module: &impl Module) {
        let slot = match section {
            Section::Left => &self.left,
            Section::Center => &self.center,
            Section::Right => &self.right,
        };
        slot.append(&module.widget());
    }

    pub fn root(&self) -> &gtk::CenterBox {
        &self.root
    }
}
