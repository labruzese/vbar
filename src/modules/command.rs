//! Command-line module: a `:` prompt and an entry filling the middle of the bar.
//!
//! Nothing is wired up behind it yet -- submitting a line just clears it. What
//! the module does have to get right is keyboard focus, which on a layer surface
//! is not the same problem as in an ordinary window.
//!
//! Opening it is the first half of that problem. A layer surface receives no key
//! events until it asks the compositor for them, so there is no keystroke vbar
//! can bind for itself: SUPER+; has to come from Hyprland, and it arrives as a
//! submap change. That also makes the command line modal the way neovim's is --
//! while the submap is active, none of your other binds fire.
//!
//!     hl.bind("SUPER + semicolon", hl.dsp.submap("vbar"))
//!     hl.define_submap("vbar", function()
//!       hl.bind("escape", hl.dsp.submap("reset"))
//!     end)

use gtk4::{self as gtk, gdk, glib, prelude::*};
use gtk4_layer_shell::{KeyboardMode, LayerShell};

use crate::bar::Module;
use crate::hypr;

/// The Hyprland submap the command line is the visible half of.
const SUBMAP: &str = "vbar";

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
        // The `:` is a mode indicator, not decoration: `focus` and `release`
        // turn it on and off with the keyboard, so it starts off. It is not
        // bound to the entry's `has-focus`, which never becomes true -- a
        // GtkEntry delegates focus to an internal GtkText -- and not to that
        // text's focus either, which GTK grabs as soon as the bar is presented,
        // long before the command line is opened.
        prompt.set_visible(false);

        let entry = gtk::Entry::new();
        entry.add_css_class("cmdline-entry");
        entry.set_has_frame(false);
        entry.set_hexpand(true);

        root.append(&prompt);
        root.append(&entry);

        // Hyprland reports submap changes as `submap>>NAME`, with an empty name
        // for the default map. That one event is both the opening and the
        // closing of the command line: the compositor owns the mode, and the bar
        // follows it.
        let watch = hypr::watch({
            let window = window.clone();
            let entry = entry.clone();
            let prompt = prompt.clone();
            move |event| match event.strip_prefix("submap>>") {
                Some(SUBMAP) => focus(&window, &entry, &prompt),
                Some(_) => release(&window, &entry, &prompt),
                None => {}
            }
        });
        if let Err(error) = watch {
            eprintln!("vbar: command line: cannot listen for SUPER+;: {error:#}");
        }

        // Escape leaves the submap, the way `:`-Escape does in neovim.
        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed({
            let window = window.clone();
            let entry = entry.clone();
            let prompt = prompt.clone();
            move |_, key, _, _| match key {
                gdk::Key::Escape => {
                    stand_down(&window, &entry, &prompt);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        entry.add_controller(keys);

        // Enter: there is no command grammar yet, so just stand down.
        entry.connect_activate({
            let window = window.clone();
            let prompt = prompt.clone();
            move |entry| stand_down(&window, entry, &prompt)
        });

        Self { root }
    }
}

impl Module for CommandLine {
    fn widget(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }
}

/// Ask Hyprland to leave the submap.
///
/// The `submap>>` event that comes back is what actually closes the command
/// line, so in the ordinary case there is nothing to do here but ask.
fn stand_down(window: &gtk::ApplicationWindow, entry: &gtk::Entry, prompt: &gtk::Label) {
    if let Err(error) = hypr::dispatch(r#"hl.dsp.submap("reset")"#) {
        // A bar left holding `Exclusive` keyboard would make the rest of the
        // desktop untypeable, so if the compositor will not close the mode for
        // us, close it here.
        eprintln!("vbar: command line: {error:#}");
        release(window, entry, prompt);
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
/// line is open, the way a launcher does. Leaving the submap gives it back.
fn focus(window: &gtk::ApplicationWindow, entry: &gtk::Entry, prompt: &gtk::Label) {
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    entry.grab_focus();
    prompt.set_visible(true);
}

/// Hand the keyboard back.
///
/// The mirror of `focus`, and reversed for the same reason: drop GTK's focus
/// while we still own the keyboard, then give the keyboard up. Revoking
/// interactivity first would have the compositor pull focus out from under GTK,
/// leaving a visibly focused entry -- and an input method still attached to it
/// -- on a surface that no longer receives keys.
fn release(window: &gtk::ApplicationWindow, entry: &gtk::Entry, prompt: &gtk::Label) {
    // Abandon the half-typed line, and the prompt with it.
    entry.set_text("");
    prompt.set_visible(false);
    // Nothing else on the bar is focusable, so clear focus rather than move it.
    // Spelled out because `Root` offers a `set_focus` of its own.
    GtkWindowExt::set_focus(window, None::<&gtk::Widget>);
    window.set_keyboard_mode(KeyboardMode::None);
}
