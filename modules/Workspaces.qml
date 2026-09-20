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

            required property HyprlandWorkspace modelData

            // Hyprland's scratchpads are the workspaces with negative ids. They
            // are not places you switch between, so the row leaves them out. A
            // Row skips invisible children when it positions, so hiding them is
            // all the filtering needed.
            visible: cell.modelData.id > 0

            text: cell.modelData.id
            // `focused` is active *and* on the focused monitor, so exactly one
            // cell lights up across all bars rather than one per monitor.
            color: cell.modelData.focused ? Theme.workspaceFocused : Theme.workspaceIdle
            font.bold: cell.modelData.focused

            leftPadding: Theme.workspacePadding
            rightPadding: Theme.workspacePadding

            TapHandler {
                // Nothing is repainted here. The compositor answers the switch
                // with an event, and that is what redraws the row -- the same
                // path a switch from a Hyprland keybind takes.
                onTapped: cell.modelData.activate()
            }
        }
    }
}
