# vbar

A neovim-like Wayland status bar for Hyprland, built on
[Quickshell](https://quickshell.org).

A statusline along the bottom edge: workspaces and a `:` command line on the
left, the clock in the middle, the focused window on the right. Gruvbox dark,
flat, monospace.

## Running

Quickshell looks for configurations under `$XDG_CONFIG_HOME/quickshell`, so
either put this repository there:

```sh
git clone <this repo> ~/.config/quickshell/vbar
qs -c vbar
```

or run it from wherever it already is:

```sh
qs -p /path/to/vbar
```

To start it with the session, add `exec-once = qs -c vbar` to your Hyprland
config.

## Hyprland configuration

The command line is opened by the compositor, not by the bar. A layer surface
receives no key events until it asks for them, so there is no keystroke vbar can
bind for itself — `SUPER+;` has to come from Hyprland, and it arrives as a
submap change. Driving it from a submap rather than an ordinary bind is also
what makes the command line modal the way neovim's is: while the submap is
active, none of your other binds fire.

```lua
hl.bind("SUPER + semicolon", hl.dsp.submap("vbar"))
hl.define_submap("vbar", function()
  hl.bind("escape", hl.dsp.submap("reset"))
end)
```

On a pre-0.55 hyprlang config the same thing reads:

```conf
bind = SUPER, semicolon, submap, vbar
submap = vbar
bind = , escape, submap, reset
submap = reset
```

The bar sets its layer-shell namespace to `vbar`, so `layerrule` matches on
that. Nothing else is required — the bar reserves its own height, so windows
tile above it rather than under it.

## Layout

```
shell.qml            One bar per monitor; Variants owns the instances.
Bar.qml              The panel window: the layer surface, the three slots,
                     and who holds the keyboard.
common/
  Theme.qml          Singleton. Every colour and metric, named once.
  BarText.qml        Shared text styling. The equivalent of a `.module` class.
modules/
  Workspaces.qml     A cell per workspace, click to switch.
  CommandLine.qml    The `:` prompt and the line you type into.
  Clock.qml          The time.
  WindowTitle.qml    The focused window, named like a buffer.
```

Files in the same directory are neighbours and need no import of each other,
which is how `BarText` reaches `Theme` and how `shell.qml` reaches `Bar`.
Across directories, `import qs.common` and `import qs.modules` are Quickshell's
root-relative imports — preferred over relative paths because `qmlls`
understands them.

## Adding a module

A module is a QML item. Start from `BarText` if it is text, give it whatever
keeps it up to date, and add it to a slot in `Bar.qml`:

```qml
// modules/Battery.qml
import QtQuick

import qs.common

BarText {
    color: Theme.foregroundBright
    text: "..."
}
```

Colours and metrics belong in `common/Theme.qml` rather than in the module, so the
palette stays in one place.

## The command line

There is no command grammar yet. `CommandLine` tracks the submap, takes the
keyboard while it is active, and on Enter emits:

```qml
signal submitted(command: string)
```

Nothing listens to it. That signal is the seam an engine plugs into — the
intended shape is a separate long-lived process owning parsing, completion,
history and execution, reached over a unix socket via `Quickshell.Io`
(`Socket`, `SocketServer`, `Process`) so that the logic is neither written in
QML nor coupled to the bar.

## Keyboard focus

Two things have to line up for the command line to receive keys, and they are
both in `Bar.qml`:

- `focusable` maps to the layer surface's *on-demand* keyboard interactivity.
  It permits the surface to hold focus without claiming it. It is bound to the
  command line's state, because a bar that always held the keyboard would make
  every other window untypeable.
- `HyprlandFocusGrab` is what actually moves focus onto the surface, and it
  hands focus back when you click away.

Neither alone is enough: without the first the compositor has nothing to give
focus to, and without the second nothing asks for it.

## Development

Creating an empty `.qmlls.ini` in this directory makes Quickshell populate it
with the import paths `qmlls` needs, which gets you completion and
go-to-definition for both Quickshell types and this configuration's own. It is
machine-specific and gitignored.

Quickshell hot-reloads on save, so there is no build step.
