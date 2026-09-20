import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland

import qs.common
import qs.modules

/// One bar, on one monitor.
///
/// The window owns two things the modules inside it cannot: the layer surface,
/// and who has the keyboard. Everything else is a module.
PanelWindow {
    id: bar

    required property ShellScreen modelData
    screen: modelData

    // What compositor rules match on -- `layerrule` in Hyprland. It has to be
    // set before the surface is created, which is why it is here and not
    // assigned later.
    WlrLayershell.namespace: "vbar"

    // Anchored to three edges: full width, height from the content, sitting at
    // the bottom the way a neovim statusline does. Anchoring three edges is
    // also what lets the exclusive zone be derived, so Hyprland tiles windows
    // above the bar rather than under it.
    anchors {
        left: true
        right: true
        bottom: true
    }

    implicitHeight: Theme.barHeight
    color: Theme.background

    // A bar that always held the keyboard would make every other window
    // untypeable, so the surface is only focusable while the command line is
    // open. `focusable` is *on-demand* interactivity -- it permits the surface
    // to hold focus without claiming it -- and the grab below is what actually
    // moves focus onto it. Neither alone is enough: without the first the
    // compositor has nothing to give focus to, and without the second nothing
    // asks for it.
    focusable: commandLine.active

    HyprlandFocusGrab {
        windows: [bar]
        active: commandLine.active
        // The compositor drops the grab when you click outside the bar. That is
        // a dismissal, and means the same thing as pressing escape.
        //
        // `cleared` also fires on the way out of an ordinary close, by which
        // point the submap is already gone -- the guard is what keeps that from
        // dispatching a second, pointless `submap reset`.
        onCleared: if (commandLine.active)
            commandLine.close()
    }

    // The bar sits at the bottom, so its rule is along the top.
    Rectangle {
        id: rule

        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
        }
        height: 1
        color: Theme.border
    }

    RowLayout {
        anchors.fill: parent
        anchors.topMargin: rule.height
        spacing: 0

        // ---- left ----------------------------------------------------------

        Row {
            Layout.fillHeight: true

            Workspaces {
                anchors.verticalCenter: parent.verticalCenter
            }

            CommandLine {
                id: commandLine

                anchors.verticalCenter: parent.verticalCenter
            }
        }

        // ---- centre --------------------------------------------------------

        // The filler takes up whatever the groups either side leave, so the
        // clock sits midway between them. It is pushed aside as the command
        // line grows rather than being overlapped by it.
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Clock {
                anchors.centerIn: parent
            }
        }

        // ---- right ---------------------------------------------------------

        Row {
            Layout.fillHeight: true

            WindowTitle {
                anchors.verticalCenter: parent.verticalCenter
            }
        }
    }
}
