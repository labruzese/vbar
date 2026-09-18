//! Command-line module: a `:` prompt and an entry filling the middle of the bar.
//!
//! Nothing is wired up behind it yet -- submitting a line just clears it. What
//! the module does have to get right is keyboard focus, which on a layer surface
//! is not the same problem as in an ordinary window.

use gtk4::{self as gtk, gdk, glib, prelude::*};
use gtk4_layer_shell::{KeyboardMode, LayerShell};

use crate::bar::Module;

pub struct CommandLine {
    root: gtk::Box,
}

impl CommandLine {
    /// `window` is the bar's layer surface. The command line needs it in order
    /// to turn keyboard interactivity on and off; see `focus` and `release`.
    pub fn new(window: &gtk::ApplicationWindow) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.add_css_class("module");
        root.add_css_class("cmdline");

        let prompt = gtk::Label::new(Some(":"));
        prompt.add_css_class("prompt");

        let entry = gtk::Entry::new();
        entry.add_css_class("cmdline-entry");
        entry.set_has_frame(false);
        entry.set_hexpand(true);
        entry.set_placeholder_text(Some("click to type a command"));

        root.append(&prompt);
        root.append(&entry);

        // Clicking the bar opens the command line. It has to be a *click*: with
        // `KeyboardMode::None` the surface receives no key events, so there is
        // no keystroke we could bind to open it.
        //
        // The gesture runs in the capture phase, i.e. before the entry sees the
        // click. That ordering matters -- the entry's own click handler grabs
        // focus, and per `focus` below the keyboard mode has to be raised first.
        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        click.connect_pressed({
            let window = window.clone();
            let entry = entry.clone();
            move |_, _, _, _| focus(&window, &entry)
        });
        root.add_controller(click);

        // Escape hands the keyboard back to the rest of the desktop.
        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed({
            let window = window.clone();
            let entry = entry.clone();
            move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    release(&window, &entry);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        });
        entry.add_controller(keys);

        // Enter: there is no command grammar yet, so just stand down.
        entry.connect_activate({
            let window = window.clone();
            move |entry| release(&window, entry)
        });

        Self { root }
    }
}

impl Module for CommandLine {
    fn widget(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }
}

/// Give the command line the keyboard.
///
/// The order is the whole point. A layer surface gets no key events at all until
/// it asks the compositor for them, so the keyboard mode has to change *first*:
/// the `Exclusive` request is what makes Hyprland send `wl_keyboard.enter` to
/// our surface. Only once the surface can hold keyboard focus does a GTK grab
/// mean anything -- grabbing first would move GTK's idea of focus while the
/// surface still had no keyboard, leaving the entry looking focused while every
/// keystroke went to whatever window Hyprland still considered focused.
///
/// `Exclusive` means the bar swallows *all* keyboard input while the command
/// line is open, the way a launcher does. Escape gives it back.
fn focus(window: &gtk::ApplicationWindow, entry: &gtk::Entry) {
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    entry.grab_focus();
}

/// Hand the keyboard back.
///
/// The mirror of `focus`, and reversed for the same reason: drop GTK's focus
/// while we still own the keyboard, then give the keyboard up. Revoking
/// interactivity first would have the compositor pull focus out from under GTK,
/// leaving a visibly focused entry -- and an input method still attached to it
/// -- on a surface that no longer receives keys.
fn release(window: &gtk::ApplicationWindow, entry: &gtk::Entry) {
    // Abandon the half-typed line, the way `:`-Escape does in neovim.
    entry.set_text("");
    // Nothing else on the bar is focusable, so clear focus rather than move it.
    // Spelled out because `Root` offers a `set_focus` of its own.
    GtkWindowExt::set_focus(window, None::<&gtk::Widget>);
    window.set_keyboard_mode(KeyboardMode::None);
}
