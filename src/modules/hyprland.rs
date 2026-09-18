//! Hyprland workspaces module: one cell per workspace, highlighting the active one.

use std::env;
use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use gtk4::{self as gtk, gio, glib, prelude::*};

use crate::bar::Module;

pub struct Workspaces {
    root: gtk::Box,
}

impl Workspaces {
    pub fn new() -> Result<Self> {
        let dir = ipc_dir()?;

        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.add_css_class("module");
        root.add_css_class("workspaces");

        let refresh = {
            let root = root.clone();
            let dir = dir.clone();
            move || {
                if let Err(error) = render(&root, &dir) {
                    eprintln!("vbar: workspaces: {error:#}");
                }
            }
        };

        refresh();
        watch_events(&dir, refresh)?;
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
fn render(root: &gtk::Box, dir: &Path) -> Result<()> {
    let mut ids = workspace_ids(&request(dir, "workspaces")?);
    let active = workspace_ids(&request(dir, "activeworkspace")?)
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
        root.append(&cell);
    }
    Ok(())
}

/// Send one command down Hyprland's request socket and read the reply.
///
/// We ask for the plain-text form rather than the `j/` (JSON) form so the bar
/// needs no JSON parser.
fn request(dir: &Path, command: &str) -> Result<String> {
    let mut socket = UnixStream::connect(dir.join(".socket.sock"))
        .context("connecting to the Hyprland request socket")?;
    socket.write_all(command.as_bytes())?;
    // Hyprland answers and then closes its end, so the reply is everything
    // up to EOF.
    let mut reply = String::new();
    socket.read_to_string(&mut reply)?;
    Ok(reply)
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

/// Watch Hyprland's event socket and call `on_event` whenever it says something.
///
/// The socket streams `event>>payload` lines, but we treat it purely as a
/// doorbell and never parse it: any event at all makes us re-query the full
/// workspace list, which is two short socket round-trips and is always
/// consistent, rather than tracking create/destroy/focus events by hand.
///
/// The socket is handed to GIO so it can be watched as an ordinary glib main
/// loop source -- the callback below is a plain main loop callback, like a
/// timeout or a GTK signal. No threads and no async runtime.
fn watch_events(dir: &Path, on_event: impl Fn() + 'static) -> Result<()> {
    let stream = UnixStream::connect(dir.join(".socket2.sock"))
        .context("connecting to the Hyprland event socket")?;
    let socket = gio::Socket::from_fd(OwnedFd::from(stream))
        .context("handing the Hyprland event socket to GIO")?;
    // The callback runs on the main loop, so it must never block. The source
    // only fires when the socket is readable, but a spurious wakeup would
    // otherwise hang the whole bar, so read non-blocking and treat `WouldBlock`
    // as "nothing to do".
    socket.set_blocking(false);

    // Spelled out because `DatagramBased` offers a `create_source` of its own.
    let source = SocketExtManual::create_source(
        &socket,
        glib::IOCondition::IN | glib::IOCondition::HUP,
        gio::Cancellable::NONE,
        Some("hyprland-events"),
        glib::Priority::DEFAULT,
        move |socket, _| {
            // The bytes are discarded; we only needed to know something happened.
            let mut chunk = [0u8; 4096];
            match socket.receive(&mut chunk[..], gio::Cancellable::NONE) {
                // Hyprland is gone: drop the watch instead of spinning on a
                // dead socket.
                Ok(0) => return glib::ControlFlow::Break,
                Ok(_) => on_event(),
                Err(error) if error.matches(gio::IOErrorEnum::WouldBlock) => {}
                Err(error) => {
                    eprintln!("vbar: hyprland event socket: {error}");
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        },
    );
    // Until it is attached the source is inert; the main context owns it (and
    // through it the socket and the callback) afterwards.
    source.attach(None);
    Ok(())
}

/// Hyprland keeps its sockets in `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE`,
/// and in `/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE` before version 0.40.
fn ipc_dir() -> Result<PathBuf> {
    let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE is unset; is Hyprland running?")?;

    env::var_os("XDG_RUNTIME_DIR")
        .map(|run| PathBuf::from(run).join("hypr").join(&signature))
        .into_iter()
        .chain([PathBuf::from("/tmp/hypr").join(&signature)])
        .find(|dir| dir.is_dir())
        .ok_or_else(|| anyhow!("no Hyprland IPC directory for instance {signature}"))
}
