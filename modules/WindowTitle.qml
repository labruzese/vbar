import QtQuick
import Quickshell.Hyprland

import qs.common

/// The focused window, named the way neovim names a buffer.
BarText {
    id: root

    readonly property string title: Hyprland.activeToplevel?.title ?? ""

    color: Theme.foregroundBright
    // Nothing focused is nothing to name, rather than a bare `>`.
    text: root.title === "" ? "" : `> ${root.title}`

    // A title is as long as the app cares to make it; the bar is not. Eliding
    // needs a width to elide against, so the text is capped here rather than
    // left to run to its implicit width.
    width: Math.min(root.implicitWidth, Theme.titleMaxWidth)
    elide: Text.ElideRight
}
