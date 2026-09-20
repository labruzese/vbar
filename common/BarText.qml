import QtQuick

/// A line of text wearing the bar's styling.
///
/// The equivalent of a `.module` class: every text module starts from this and
/// overrides only what makes it that module. Colour and padding are ordinary
/// properties, so a module that wants different ones just sets them.
Text {
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontPixelSize
    color: Theme.foreground

    leftPadding: Theme.modulePadding
    rightPadding: Theme.modulePadding
    verticalAlignment: Text.AlignVCenter

    // The bar is 26px of small monospace text. Hinting it against the pixel
    // grid is the difference between crisp and smeared at this size.
    renderType: Text.NativeRendering
}
