import QtQuick
import Quickshell.Hyprland

import qs.common

/// A `:` prompt and the line you type into it.
///
/// Opening it is the compositor's decision, not ours. A layer surface receives
/// no key events until it asks for them, so there is no keystroke the bar can
/// bind for itself: SUPER+; comes from Hyprland and arrives here as a submap
/// change. Driving it from a submap rather than an ordinary bind is also what
/// makes the command line modal the way neovim's is -- while the submap is
/// active, none of your other binds fire.
///
///     hl.bind("SUPER + semicolon", hl.dsp.submap("vbar"))
///     hl.define_submap("vbar", function()
///       hl.bind("escape", hl.dsp.submap("reset"))
///     end)
///
/// There is no command grammar yet. Submitting a line emits `submitted` and
/// closes; that signal is the seam an engine plugs into.
Row {
    id: root

    /// The Hyprland submap this command line is the visible half of.
    readonly property string submapName: "vbar"

    /// The submap Hyprland is currently in; empty for the default map. Written
    /// only by the event handler below -- the compositor owns this, and the bar
    /// follows it.
    property string submap: ""

    /// Whether the command line is open and expecting keystrokes. `Bar` reads
    /// this to decide whether the surface may hold the keyboard.
    readonly property bool active: root.submap === root.submapName

    /// A line was accepted. Nothing listens yet.
    signal submitted(command: string)

    /// Ask Hyprland to leave the submap.
    ///
    /// The `submap>>` event that comes back is what actually closes the command
    /// line, so in the ordinary case there is nothing to do here but ask.
    function close(): void {
        // Hyprland 0.55 moved its config to Lua, and dispatchers with it. Both
        // spellings are one string to this socket, so pick by what is running
        // rather than by what we were built against.
        Hyprland.dispatch(Hyprland.usingLua ? 'hl.dsp.submap("reset")' : "submap reset");
    }

    leftPadding: Theme.modulePadding
    rightPadding: Theme.modulePadding
    spacing: Theme.commandPromptGap

    onActiveChanged: {
        if (root.active) {
            input.forceActiveFocus();
        } else {
            // Abandon the half-typed line, and the focus with it. Nothing else
            // on the bar is focusable, so this clears focus rather than moving
            // it anywhere.
            input.text = "";
            input.focus = false;
        }
    }

    Connections {
        target: Hyprland

        // Hyprland reports submap changes as `submap>>NAME`, with an empty name
        // for the default map. That one event is both the opening and the
        // closing of the command line.
        function onRawEvent(event) {
            if (event.name === "submap")
                root.submap = event.data;
        }
    }

    BarText {
        anchors.verticalCenter: parent.verticalCenter

        text: ":"
        color: Theme.prompt
        leftPadding: 0
        rightPadding: 0

        // The `:` is a mode indicator, not decoration.
        visible: root.active
    }

    TextInput {
        id: input

        anchors.verticalCenter: parent.verticalCenter

        // Fixed, and reserved whether or not the line is open, so that opening
        // it does not shunt the rest of the bar sideways.
        width: Theme.commandWidth

        font.families: Theme.fontFamilies
        font.pixelSize: Theme.fontPixelSize
        renderType: Text.NativeRendering

        color: Theme.foregroundInput
        selectionColor: Theme.selection
        selectedTextColor: Theme.fg0
        selectByMouse: true

        // A steady bar, like the one neovim leaves in its command line. The
        // default caret would be drawn in the text colour; this keeps the
        // accent the stylesheet always had.
        cursorDelegate: Rectangle {
            width: 1
            color: Theme.cursor
        }

        onAccepted: {
            root.submitted(input.text);
            root.close();
        }

        // Escape leaves the submap, the way `:`-Escape does in neovim. The bind
        // inside the submap does the same thing; this covers the case where the
        // command line has the keyboard and Hyprland's bind does not fire.
        Keys.onEscapePressed: event => {
            root.close();
            event.accepted = true;
        }
    }
}
