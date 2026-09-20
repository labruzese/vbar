pragma Singleton

import Quickshell

/// vbar -- gruvbox dark. Flat: no rounding, no gradients, no shadows.
///
/// Every colour and metric the bar uses is named once, here. Modules refer to
/// the *roles* below rather than to the palette directly, so restyling is a
/// change to this file and never a grep for a hex code.
Singleton {
    id: root

    // ---- gruvbox dark ------------------------------------------------------

    readonly property color bg0Hard: "#1d2021"
    readonly property color bg1: "#3c3836"
    readonly property color bg2: "#504945"
    readonly property color fg0: "#fbf1c7"
    readonly property color fg1: "#ebdbb2"
    readonly property color fg2: "#d5c4a1"
    readonly property color fg4: "#a89984"
    readonly property color yellow: "#fabd2f"
    readonly property color blue: "#83a598"
    readonly property color orange: "#fe8019"

    // ---- roles -------------------------------------------------------------

    readonly property color background: root.bg0Hard
    readonly property color border: root.bg1

    /// The default for a text module: present, but not competing with content.
    readonly property color foreground: root.fg4
    /// A module that is saying something specific -- the clock, a window name.
    readonly property color foregroundBright: root.fg2
    /// Text being typed, and the selection behind it.
    readonly property color foregroundInput: root.fg1
    readonly property color selection: root.bg2

    /// Workspaces you are not on. Deliberately close to the background: the row
    /// should read as one figure, like neovim's ruler, not a strip of buttons.
    readonly property color workspaceIdle: root.bg2
    readonly property color workspaceFocused: root.yellow

    readonly property color prompt: root.blue
    readonly property color cursor: root.orange

    // ---- metrics -----------------------------------------------------------

    readonly property int barHeight: 26
    /// Horizontal breathing room around a text module.
    readonly property int modulePadding: 10
    /// Workspace cells sit tighter than that, so the numbers group.
    readonly property int workspacePadding: 4
    /// Between the `:` and what you are typing after it.
    readonly property int commandPromptGap: 6

    /// Qt tries these in order, so the Nerd Font is used where it is installed
    /// and the plain family where it is not.
    readonly property var fontFamilies: ["JetBrainsMono Nerd Font", "JetBrains Mono Nerd Font", "JetBrains Mono", "monospace"]
    readonly property int fontPixelSize: 13

    /// How much of the bar a single window title may take before it is elided.
    readonly property int titleMaxWidth: 420
    /// The command line's typing area. Fixed, so the modules either side of it
    /// do not move as you type.
    readonly property int commandWidth: 420
}
