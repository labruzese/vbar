//! Window title module: the focused window, named the way neovim names a buffer.

use anyhow::Result;
use gtk4::{self as gtk, pango, prelude::*};

use crate::bar::{Module, label};
use crate::hypr;

pub struct WindowTitle {
    label: gtk::Label,
}

impl WindowTitle {
    pub fn new() -> Result<Self> {
        let label = label("window");
        // A title is as long as the app cares to make it; the bar is not.
        label.set_max_width_chars(48);
        label.set_ellipsize(pango::EllipsizeMode::End);

        let refresh = {
            let label = label.clone();
            move || match title() {
                Ok(Some(title)) => label.set_text(&format!("win/{title}.win")),
                // Nothing focused, so there is nothing to name.
                Ok(None) => label.set_text(""),
                Err(error) => {
                    eprintln!("vbar: window title: {error:#}");
                    label.set_text("");
                }
            }
        };

        refresh();
        // As in `workspaces`: any event is a doorbell to re-ask, which covers
        // focus changes and a window renaming itself alike.
        hypr::watch(move |_| refresh())?;
        Ok(Self { label })
    }
}

impl Module for WindowTitle {
    fn widget(&self) -> gtk::Widget {
        self.label.clone().upcast()
    }
}

/// The focused window's title, or `None` when nothing is focused.
///
/// `activewindow` answers with one indented `property: value` line per
/// property, so the title is the `title:` line. `initialTitle:` is a separate
/// property and does not collide once the line is trimmed.
fn title() -> Result<Option<String>> {
    let reply = hypr::request("activewindow")?;
    Ok(reply
        .lines()
        .filter_map(|line| line.trim().strip_prefix("title:"))
        .map(|title| title.trim().to_owned())
        // A window that has not named itself yet gets no name here either,
        // rather than a bare `win/.win`.
        .find(|title| !title.is_empty()))
}
