pragma ComponentBehavior: Bound

import QtQuick
import Quickshell.Hyprland

import qs.common

/// One cell per workspace, highlighting the one you are on.
///
/// Nothing here tracks compositor state by hand: `Hyprland.workspaces` is a live
/// model, so workspaces appearing, going away and changing focus are all just
/// the row re-rendering.
Row {
    leftPadding: Theme.workspacePadding
    rightPadding: Theme.modulePadding

    Repeater {
        model: Hyprland.workspaces

        delegate: BarText {
            id: cell

            /// A `HyprlandWorkspace`, held as `var` rather than declared as one
            /// so that whatever the model hands over binds without a coercion
            /// that can only fail silently.
            required property var modelData

            // A delegate outlives its workspace: when one goes away the
            // bindings below re-evaluate once with `modelData` already null.
            // Reading the two fields through guards here means that frame costs
            // a default instead of four TypeErrors, and means it is read once
            // per change rather than once per binding.
            readonly property int workspaceId: cell.modelData?.id ?? -1
            readonly property bool focused: cell.modelData?.focused ?? false

            // Hyprland's scratchpads are the workspaces with negative ids. They
            // are not places you switch between, so the row leaves them out --
            // as is a cell whose workspace has just gone. A Row skips invisible
            // children when it positions, so hiding them is all the filtering
            // needed.
            visible: cell.workspaceId > 0

            text: cell.workspaceId
            // `focused` is active *and* on the focused monitor, so exactly one
            // cell lights up across all bars rather than one per monitor.
            color: cell.focused ? Theme.workspaceFocused : Theme.workspaceIdle
            font.bold: cell.focused

            leftPadding: Theme.workspacePadding
            rightPadding: Theme.workspacePadding

            TapHandler {
                // Nothing is repainted here. The compositor answers the switch
                // with an event, and that is what redraws the row -- the same
                // path a switch from a Hyprland keybind takes.
                //
                // Optional on the same grounds: a tap landing in the frame a
                // workspace is removed would otherwise throw.
                onTapped: cell.modelData?.activate()
            }
        }
    }
}
