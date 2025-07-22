//! CSS styles for the application
//! This module contains all CSS styling definitions used throughout the application

use gtk::{CssProvider, gdk::Display};

/// Applies the application's CSS styling
pub fn apply_application_styles() {
    let provider = CssProvider::new();
    provider.load_from_data(&get_application_css());
    
    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Sets up the icon theme for the application
pub fn setup_icon_theme() {
    if let Some(display) = Display::default() {
        let icon_theme = gtk::IconTheme::for_display(&display);
        icon_theme.add_resource_path("/com/cali666/lsfg-vk-ui/icons");
    }
}

/// Returns the application's CSS as a string
fn get_application_css() -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}{}{}",
        get_button_styles(),
        get_sidebar_styles(),
        get_profile_styles(),
        get_settings_styles(),
        get_feedback_styles(),
        get_process_picker_styles(),
        get_animation_styles(),
        get_libadwaita_enhancements(),
        get_toast_styles(),
        get_preferences_styles(),
        get_icon_styles()
    )
}

/// Button-related styles
fn get_button_styles() -> &'static str {
    r#"
        .settings-icon-button {
            font-size: 1.4rem;
        }

        .circular-button {
            min-width: 32px;
            min-height: 32px;
            padding: 4px;
        }

        .icon-button {
            min-width: 32px;
            min-height: 32px;
            padding: 6px;
        }

        .linked-button-box {
            margin-top: 12px;
            margin-bottom: 12px;
        }
    "#
}

/// Sidebar-related styles
fn get_sidebar_styles() -> &'static str {
    r#"
        .sidebar {
            background-color: @theme_bg_color;
        }
        
        .sidebar-content {
            background-color: shade(@theme_bg_color, 0.95);
            color: @theme_fg_color;
            padding: 12px;
        }
    "#
}

/// Profile row styles
fn get_profile_styles() -> &'static str {
    r#"
        .profile-row {
            padding: 8px 12px;
        }

        .profile-row:hover {
            background-color: alpha(@theme_fg_color, 0.1);
        }

        .profile-row:selected {
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }

        .profile-actions {
            opacity: 0.7;
            transition: opacity 200ms ease;
        }

        .profile-row:hover .profile-actions {
            opacity: 1.0;
        }
    "#
}

/// Settings window styles
fn get_settings_styles() -> &'static str {
    r#"
        .settings-group {
            margin: 12px;
            padding: 12px;
            border-radius: 8px;
            background-color: alpha(@theme_bg_color, 0.5);
        }

        .settings-row {
            padding: 8px 0;
        }
    "#
}

/// Feedback and notification styles
fn get_feedback_styles() -> &'static str {
    r#"
        .feedback-label {
            color: @success_color;
            font-weight: bold;
            animation: fade-in-out 2s ease-in-out;
        }
    "#
}

/// Process picker window styles
fn get_process_picker_styles() -> &'static str {
    r#"
        .process-picker-window {
            background-color: @theme_bg_color;
        }

        .process-list-item {
            padding: 8px 12px;
            border-radius: 4px;
            margin: 2px 0;
        }

        .process-list-item:hover {
            background-color: alpha(@theme_fg_color, 0.1);
        }

        .process-list-item:selected {
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }
    "#
}

/// Animation definitions
fn get_animation_styles() -> &'static str {
    r#"
        @keyframes fade-in-out {
            0% { opacity: 0; }
            20% { opacity: 1; }
            80% { opacity: 1; }
            100% { opacity: 0; }
        }

        @keyframes slide-in {
            from {
                opacity: 0;
                transform: translateY(-10px);
            }
            to {
                opacity: 1;
                transform: translateY(0);
            }
        }

        @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.7; }
            100% { opacity: 1; }
        }
    "#
}

/// Libadwaita-specific enhancements
fn get_libadwaita_enhancements() -> &'static str {
    r#"
        /* Enhanced libadwaita integration */
        .adw-preferences-group {
            margin: 18px 0;
        }

        .adw-preferences-row {
            padding: 12px 16px;
        }

        .adw-action-row {
            min-height: 56px;
        }

        .adw-entry-row {
            min-height: 56px;
        }

        /* Toast styling */
        .adw-toast {
            border-radius: 8px;
            margin: 12px;
        }
    "#
}

/// Toast notification styles
fn get_toast_styles() -> &'static str {
    r#"
        .toast-success {
            background-color: @success_color;
            color: @success_fg_color;
        }

        .toast-error {
            background-color: @error_color;
            color: @error_fg_color;
        }

        .toast-warning {
            background-color: @warning_color;
            color: @warning_fg_color;
        }
    "#
}

/// Preferences window styles
fn get_preferences_styles() -> &'static str {
    r#"
        .preferences-window {
            background-color: @window_bg_color;
        }

        .preferences-page {
            padding: 24px;
        }

        .preferences-group-header {
            font-weight: bold;
            margin-bottom: 12px;
        }
    "#
}

/// Icon-related styles
fn get_icon_styles() -> &'static str {
    r#"
        /* Ensure symbolic icons use proper colors */
        .symbolic {
            -gtk-icon-style: symbolic;
        }

        /* Icon button hover effects */
        .icon-button:hover {
            background-color: alpha(@theme_fg_color, 0.1);
        }

        .icon-button:active {
            background-color: alpha(@theme_fg_color, 0.2);
        }
    "#
}

/// CSS class constants for consistent styling
pub mod css_classes {
    pub const CIRCULAR_BUTTON: &[&str] = &["flat", "circular"];
    pub const DESTRUCTIVE_CIRCULAR_BUTTON: &[&str] = &["flat", "circular", "destructive-action"];
    pub const ICON_BUTTON: &[&str] = &["flat", "icon-button"];
    pub const SUGGESTED_ACTION: &[&str] = &["suggested-action"];
    pub const DESTRUCTIVE_ACTION: &[&str] = &["destructive-action"];
    pub const PROFILE_ROW: &[&str] = &["profile-row"];
    pub const PROFILE_ACTIONS: &[&str] = &["profile-actions"];
    pub const SETTINGS_GROUP: &[&str] = &["settings-group"];
    pub const PROCESS_LIST_ITEM: &[&str] = &["process-list-item"];
}