//! Clock module: a label repainted once a second.

use gtk4::{self as gtk, glib, prelude::*};

use crate::bar::{Module, label};

pub struct Clock {
    label: gtk::Label,
}

impl Clock {
    /// `format` is a `g_date_time_format` (strftime-like) string.
    pub fn new(format: &str) -> Self {
        let label = label("clock");
        let format = format.to_owned();

        let tick = {
            let label = label.clone();
            move || {
                match glib::DateTime::now_local().and_then(|now| now.format(&format)) {
                    Ok(text) => label.set_text(&text),
                    // A clock that cannot format the time is not worth taking
                    // the bar down for: show a placeholder and keep ticking.
                    Err(error) => {
                        eprintln!("vbar: clock: {error}");
                        label.set_text("--:--");
                    }
                }
            }
        };

        // Paint once up front; the first timeout is a second away.
        tick();
        glib::timeout_add_seconds_local(1, move || {
            tick();
            glib::ControlFlow::Continue
        });

        Self { label }
    }
}

impl Module for Clock {
    fn widget(&self) -> gtk::Widget {
        self.label.clone().upcast()
    }
}
