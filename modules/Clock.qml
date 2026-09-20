import QtQuick
import Quickshell

import qs.common

/// The time, in the corner of your eye.
BarText {
    id: root

    /// A `QDateTime` format string.
    property string format: "ddd dd MMM hh:mm AP"

    color: Theme.foregroundBright
    text: Qt.formatDateTime(clock.date, root.format)

    SystemClock {
        id: clock

        // The format above stops at minutes, and so does the clock. Asking for
        // second precision would wake the process sixty times for every visible
        // change.
        precision: SystemClock.Minutes
    }
}
