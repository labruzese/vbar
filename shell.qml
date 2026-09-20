//! vbar -- a neovim-like Wayland status bar for Hyprland.
//!
//! Run it with `qs -c vbar` once this directory is at
//! `$XDG_CONFIG_HOME/quickshell/vbar`, or `qs -p .` from here.

import Quickshell

ShellRoot {
    // One bar per monitor. `Variants` owns the instances: plug a display in and
    // a bar appears on it, unplug it and the bar goes with it, with no signal
    // handling of our own.
    Variants {
        model: Quickshell.screens
        delegate: Bar {}
    }
}
