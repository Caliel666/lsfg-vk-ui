//! UI Helper functions - Legacy module
//! Most functions have been moved to ui_components.rs for better organization
//! This module is kept for backward compatibility

use gtk::glib;

/// Creates a widget from builder with error handling
#[allow(dead_code)]
#[allow(dead_code)]
#[allow(dead_code)]
pub fn get_widget_from_builder<T: glib::IsA<glib::Object>>(
    builder: &gtk::Builder,
    widget_id: &str,
) -> Result<T, String> {
    builder
        .object(widget_id)
        .ok_or_else(|| format!("Could not get {} from builder", widget_id))
}