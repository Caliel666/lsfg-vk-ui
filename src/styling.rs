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
        ".settings-icon-button {{
            font-size: 1.4rem;
        }}

        .sidebar {{
            background-color: @theme_bg_color;
        }}
        
        .sidebar-content {{
            background-color: shade(@theme_bg_color, {});
            color: @theme_fg_color;
            padding: 12px;
        }}

        .linked-button-box {{
            margin-top: 12px;
            margin-bottom: 12px;
        }}

        .profile-row {{
            padding: 8px 12px;
        }}

        .profile-row:hover {{
            background-color: alpha(@theme_fg_color, 0.1);
        }}

        .profile-row:selected {{
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }}

        .profile-actions {{
            opacity: 0.7;
            transition: opacity 200ms ease;
        }}

        .profile-row:hover .profile-actions {{
            opacity: 1.0;
        }}

        .circular-button {{
            min-width: 32px;
            min-height: 32px;
            padding: 4px;
        }}

        .icon-button {{
            min-width: 32px;
            min-height: 32px;
            padding: 6px;
        }}

        .feedback-label {{
            color: @success_color;
            font-weight: bold;
            animation: fade-in-out 2s ease-in-out;
        }}

        @keyframes fade-in-out {{
            0% {{ opacity: 0; }}
            20% {{ opacity: 1; }}
            80% {{ opacity: 1; }}
            100% {{ opacity: 0; }}
        }}

        .settings-group {{
            margin: 12px;
            padding: 12px;
            border-radius: 8px;
            background-color: alpha(@theme_bg_color, 0.5);
        }}

        .settings-row {{
            padding: 8px 0;
        }}

        .process-picker-window {{
            background-color: @theme_bg_color;
        }}

        .process-list-item {{
            padding: 8px 12px;
            border-radius: 4px;
            margin: 2px 0;
        }}

        .process-list-item:hover {{
            background-color: alpha(@theme_fg_color, 0.1);
        }}

        .process-list-item:selected {{
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }}",
        0.95 // Sidebar shade factor
    )
}

/// CSS class constants for consistent styling
pub mod css_classes {
    pub const CIRCULAR_BUTTON: &[&str] = &["flat", "circular"];
    pub const DESTRUCTIVE_CIRCULAR_BUTTON: &[&str] = &["flat", "circular", "destructive-action"];
}
