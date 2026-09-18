//! Hyprland workspaces module: one cell per workspace, highlighting the active one.

use anyhow::Result;
use gtk4::{self as gtk, prelude::*};

use crate::bar::Module;
use crate::hypr;

pub struct Workspaces {
    root: gtk::Box,
}

impl Workspaces {
    pub fn new() -> Result<Self> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.add_css_class("module");
        root.add_css_class("workspaces");

        let refresh = {
            let root = root.clone();
            move || {
                if let Err(error) = render(&root) {
                    eprintln!("vbar: workspaces: {error:#}");
                }
            }
        };

        refresh();
        // Any event at all is treated as a doorbell: re-querying the full list
        // is two short round-trips and is always consistent, where tracking
        // create/destroy/focus events by hand would not be.
        hypr::watch(move |_| refresh())?;
        Ok(Self { root })
    }
}

impl Module for Workspaces {
    fn widget(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }
}

/// Rebuild the row of workspace cells from Hyprland's current state.
///
/// Throwing the labels away and making new ones on every event is crude, but at
/// ten-ish workspaces it is cheaper than the bookkeeping needed to diff them,
/// and it cannot drift out of sync with the compositor.
fn render(root: &gtk::Box) -> Result<()> {
    let mut ids = workspace_ids(&hypr::request("workspaces")?);
    let active = workspace_ids(&hypr::request("activeworkspace")?)
        .first()
        .copied();
    ids.sort_unstable();

    while let Some(child) = root.first_child() {
        root.remove(&child);
    }
    for id in ids {
        let cell = gtk::Label::new(Some(&id.to_string()));
        cell.add_css_class("workspace");
        if Some(id) == active {
            cell.add_css_class("active");
        }
        // A label has no notion of being pressed, so the click comes from a
        // gesture attached to it.
        let click = gtk::GestureClick::new();
        click.connect_pressed(move |_, _, _, _| switch_to(id));
        cell.add_controller(click);
        root.append(&cell);
    }
    Ok(())
}

/// Ask Hyprland to focus a workspace.
///
/// Nothing is repainted here: the compositor answers the switch with an event,
/// and that is what redraws the row -- the same path a switch from a Hyprland
/// keybind takes.
fn switch_to(id: i32) {
    if let Err(error) = hypr::dispatch(&format!("hl.dsp.focus({{ workspace = \"{id}\" }})")) {
        eprintln!("vbar: workspaces: {error:#}");
    }
}

/// Both `workspaces` and `activeworkspace` introduce each workspace with a
/// header line like `workspace ID 3 (3) on monitor DP-1:`, so one parser covers
/// both replies.
///
/// Special workspaces (Hyprland's scratchpads) are the ones with negative IDs;
/// they are not places you switch between, so the bar leaves them out.
fn workspace_ids(reply: &str) -> Vec<i32> {
    reply
        .lines()
        .filter_map(|line| line.strip_prefix("workspace ID "))
        .filter_map(|rest| rest.split_whitespace().next()?.parse().ok())
        .filter(|id| *id > 0)
        .collect()
}
