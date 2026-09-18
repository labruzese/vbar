//! Hyprland IPC: the two sockets every Hyprland-shaped part of vbar talks to.
//!
//! `.socket.sock` takes one request per connection and answers in plain text.
//! `.socket2.sock` streams `event>>payload` lines for as long as it is held
//! open, and is how the bar hears about anything that changes.

use std::env;
use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use gtk4::{gio, glib, prelude::*};

/// Send one request and read the reply.
///
/// We ask for the plain-text form rather than the `j/` (JSON) form so the bar
/// needs no JSON parser. Hyprland serves this socket synchronously and blocks
/// the rest of the compositor while it does, so the connection is opened, used
/// and dropped here rather than kept around.
pub fn request(command: &str) -> Result<String> {
    let mut socket = UnixStream::connect(dir()?.join(".socket.sock"))
        .context("connecting to the Hyprland request socket")?;
    socket.write_all(command.as_bytes())?;
    // Hyprland answers and then closes its end, so the reply is everything up
    // to EOF.
    let mut reply = String::new();
    socket.read_to_string(&mut reply)?;
    Ok(reply)
}

/// Run a dispatcher, spelled as the Lua expression Hyprland 0.55+ wants -- for
/// instance `hl.dsp.submap("reset")`.
///
/// Hyprland answers `ok`, or the error it raised; neither arrives as anything
/// the socket itself would call a failure, so the reply has to be read.
pub fn dispatch(lua: &str) -> Result<()> {
    let reply = request(&format!("dispatch {lua}"))?;
    match reply.trim() {
        "ok" => Ok(()),
        error => Err(anyhow!("dispatch {lua}: {error}")),
    }
}

/// Call `on_event` with each `event>>payload` line Hyprland emits.
///
/// The socket is handed to GIO so it can be watched as an ordinary glib main
/// loop source -- the callback below is a plain main loop callback, like a
/// timeout or a GTK signal. No threads and no async runtime.
///
/// Every caller gets its own connection. Hyprland is happy to serve several,
/// and one stream per watcher is less machinery than handing one around.
pub fn watch(on_event: impl Fn(&str) + 'static) -> Result<()> {
    let stream = UnixStream::connect(dir()?.join(".socket2.sock"))
        .context("connecting to the Hyprland event socket")?;
    let socket = gio::Socket::from_fd(OwnedFd::from(stream))
        .context("handing the Hyprland event socket to GIO")?;
    // The callback runs on the main loop, so it must never block. The source
    // only fires when the socket is readable, but a spurious wakeup would
    // otherwise hang the whole bar, so read non-blocking and treat `WouldBlock`
    // as "nothing to do".
    socket.set_blocking(false);

    let mut pending = String::new();
    // Spelled out because `DatagramBased` offers a `create_source` of its own.
    let source = SocketExtManual::create_source(
        &socket,
        glib::IOCondition::IN | glib::IOCondition::HUP,
        gio::Cancellable::NONE,
        Some("hyprland-events"),
        glib::Priority::DEFAULT,
        move |socket, _| {
            let mut chunk = [0u8; 4096];
            match socket.receive(&mut chunk[..], gio::Cancellable::NONE) {
                // Hyprland is gone: drop the watch instead of spinning on a
                // dead socket.
                Ok(0) => return glib::ControlFlow::Break,
                Ok(read) => {
                    pending.push_str(&String::from_utf8_lossy(&chunk[..read]));
                    // Events are newline terminated and a read can land
                    // mid-line, so hand on the whole lines and keep the
                    // remainder for the next wakeup.
                    while let Some(end) = pending.find('\n') {
                        let rest = pending.split_off(end + 1);
                        on_event(pending.trim_end());
                        pending = rest;
                    }
                }
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
fn dir() -> Result<PathBuf> {
    let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE is unset; is Hyprland running?")?;

    env::var_os("XDG_RUNTIME_DIR")
        .map(|run| PathBuf::from(run).join("hypr").join(&signature))
        .into_iter()
        .chain([PathBuf::from("/tmp/hypr").join(&signature)])
        .find(|dir| dir.is_dir())
        .ok_or_else(|| anyhow!("no Hyprland IPC directory for instance {signature}"))
}
