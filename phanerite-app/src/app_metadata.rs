//! Application identity shared by the launcher and native build resources.

/// User-visible application name in the operating system.
pub const APP_NAME: &str = "Phanerite";

/// Title shared by the native window and the custom title bar.
pub const APP_WINDOW_TITLE: &str = "Phanerite Launcher";

/// Machine-readable ID for the application.
///
/// This should be globally unique, usually by using Reverse Domain Name
/// Notation. Keep the desktop entry and bundle identifier in sync with it.
///
/// PLEASE CHANGE THIS IF YOU DISTRIBUTE A MODIFIED VERSION OF PHANERITE.
pub const APP_ID: &str = "org.feniota.phanerite";
