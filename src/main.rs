//! vbar -- a neovim like Wayland status bar for Hyprland.

mod bar;
mod hypr;
mod modules;

use std::path::PathBuf;

use anyhow::{Result, bail};
use gtk4::{self as gtk, gdk, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use bar::{Bar, Section};
use modules::{clock::Clock, command::CommandLine, window::WindowTitle, workspaces::Workspaces};

const APP_ID: &str = "dev.abruzese.vbar";

const DEFAULT_CSS: &str = include_str!("../style.css");

fn main() -> Result<()> {
    let app = gtk::Application::builder().application_id(APP_ID).build();

    // The stylesheet has to be on the display before any widget is built, so it
    // goes in `startup` rather than `activate`.
    app.connect_startup(|_| load_css());
    app.connect_activate(|app| {
        if let Err(error) = build(app) {
            eprintln!("vbar: {error:#}");
            app.quit();
        }
    });

    // vbar takes no arguments of its own yet, so don't hand GTK our argv.
    let code = app.run_with_args::<&str>(&[]);
    if code != glib::ExitCode::SUCCESS {
        bail!("vbar exited with a failure status");
    }
    Ok(())
}

fn build(app: &gtk::Application) -> Result<()> {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .name("vbar")
        .build();

    // Layer-shell setup has to happen before the window is first presented: it
    // turns the toplevel into a `zwlr_layer_surface_v1`, and that is not a
    // decision that can be made once the surface is mapped.
    window.init_layer_shell();
    // The namespace is what compositor rules match on (`layerrule` in Hyprland).
    window.set_namespace(Some("vbar"));
    window.set_layer(Layer::Top);
    // Anchored to three edges: full width, height from the content, sitting at
    // the bottom the way a neovim statusline does.
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Bottom, true);
    // Reserve that height so Hyprland tiles windows above the bar, not under it.
    window.auto_exclusive_zone_enable();
    // Start inert. A bar holding keyboard focus would make every other window
    // untypeable; the command line raises this only while it is in use.
    window.set_keyboard_mode(KeyboardMode::None);

    let bar = Bar::new();
    bar.push(Section::Center, &Clock::new("%a %d %b %I:%M %p"));

    match WindowTitle::new() {
        Ok(title) => bar.push(Section::Right, &title),
        Err(error) => eprintln!("vbar: window title module disabled: {error:#}"),
    }

    match Workspaces::new() {
        Ok(workspaces) => bar.push(Section::Left, &workspaces),
        // Not being under Hyprland costs you one module, not the bar.
        Err(error) => eprintln!("vbar: workspaces module disabled: {error:#}"),
    }

    bar.push(Section::Left, &CommandLine::new(&window));


    window.set_child(Some(bar.root()));
    window.present();
    Ok(())
}

/// Load `style.css` from the user's config directory if they have one, else the
/// copy compiled into the binary.
fn load_css() {
    let provider = gtk::CssProvider::new();

    // Parse errors arrive on a signal rather than as a `Result`, and GTK simply
    // skips the offending rule. Connect before loading so they are at least
    // visible.
    provider.connect_parsing_error(|_, section, error| {
        eprintln!("vbar: style.css: {section}: {error}");
    });

    match user_stylesheet() {
        Some(path) => provider.load_from_path(&path),
        None => provider.load_from_string(DEFAULT_CSS),
    }

    match gdk::Display::default() {
        Some(display) => gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        ),
        None => eprintln!("vbar: no display to style; continuing unstyled"),
    }
}

/// `$XDG_CONFIG_HOME/vbar/style.css`, falling back to `~/.config/vbar/style.css`.
fn user_stylesheet() -> Option<PathBuf> {
    let path = glib::user_config_dir().join("vbar").join("style.css");
    path.is_file().then_some(path)
}
