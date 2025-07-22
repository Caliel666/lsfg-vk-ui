use gtk::prelude::*;
use gtk::{glib, Builder};
use libadwaita::ApplicationWindow;
use std::cell::RefCell;
use std::rc::Rc;

// Import modules
mod config;
mod app_state;
mod utils;
mod settings_window;
mod ui_helpers;
mod ui_components;
mod profile_manager;
mod signal_handlers;
mod styling;
mod styles;

use config::{load_config, Config, OrderedGlobalConfig};
use app_state::AppState;
use ui_components::WidgetUtils;
use signal_handlers::{
    connect_profile_signal_handlers, connect_save_button_handler,
    connect_sidebar_row_activated_handler, connect_create_profile_button_handler
};
use styles::{apply_application_styles, setup_icon_theme};

fn main() -> glib::ExitCode {
    let application = libadwaita::Application::builder()
        .application_id("com.cali666.lsfg-vk-ui")
        .build();
    
    // Set the desktop file name for proper GNOME integration
    glib::set_application_name("LSFG-VK UI");
    glib::set_prgname(Some("lsfg-vk-ui"));

    application.connect_startup(|_app| {
        apply_application_styles();
        setup_icon_theme();
    });

    application.connect_activate(|app| {
        if let Err(e) = setup_main_window(app) {
            eprintln!("Failed to setup main window: {}", e);
        }
    });

    application.run()
}

fn setup_main_window(app: &libadwaita::Application) -> Result<(), Box<dyn std::error::Error>> {
    // Load initial configuration
    let initial_config = load_config().unwrap_or_else(|e| {
        eprintln!("Error loading config: {}", e);
        Config {
            version: 1,
            ordered_global: OrderedGlobalConfig { global: None },
            game: Vec::new()
        }
    });

    // Load UI from .ui file
    let ui_bytes = include_bytes!("../resources/ui.ui");
    let builder = Builder::from_string(std::str::from_utf8(ui_bytes)?);

    // Get widgets from builder
    let widgets = extract_widgets_from_builder(&builder)?;
    
    // Set up main window
    widgets.main_window.set_application(Some(app));
    widgets.main_window.set_icon_name(Some("com.cali666.lsfg-vk-ui"));

    // Create save button
    let save_button = gtk::Button::builder()
        .label("Save Changes")
        .halign(gtk::Align::End)
        .margin_end(12)
        .margin_bottom(12)
        .build();
    widgets.main_settings_box.append(&save_button);

    // Initialize application state
    let app_state = Rc::new(RefCell::new(AppState {
        config: initial_config,
        selected_profile_index: None,
        main_window: widgets.main_window.clone(),
        sidebar_list_box: widgets.sidebar_list_box.clone(),
        multiplier_dropdown: widgets.multiplier_dropdown.clone(),
        flow_scale_entry: widgets.flow_scale_entry.clone(),
        performance_mode_switch: widgets.performance_mode_switch.clone(),
        hdr_mode_switch: widgets.hdr_mode_switch.clone(),
        experimental_present_mode_dropdown: widgets.experimental_present_mode_dropdown.clone(),
        save_button: save_button.clone(),
        main_settings_box: widgets.main_settings_box.clone(),
        multiplier_dropdown_handler_id: None,
        flow_scale_entry_handler_id: None,
        performance_mode_switch_handler_id: None,
        hdr_mode_switch_handler_id: None,
        experimental_present_mode_dropdown_handler_id: None,
    }));

    // Connect all signal handlers
    connect_signal_handlers(&widgets, &app_state, &save_button);

    // Initialize UI
    initialize_ui(&app_state);

    widgets.main_window.present();
    Ok(())
}

struct AppWidgets {
    main_window: ApplicationWindow,
    settings_button: gtk::Button,
    sidebar_list_box: gtk::ListBox,
    create_profile_button: gtk::Button,
    multiplier_dropdown: gtk::DropDown,
    flow_scale_entry: gtk::Entry,
    performance_mode_switch: gtk::Switch,
    hdr_mode_switch: gtk::Switch,
    experimental_present_mode_dropdown: gtk::DropDown,
    main_settings_box: gtk::Box,
}

fn extract_widgets_from_builder(builder: &Builder) -> Result<AppWidgets, String> {
    Ok(AppWidgets {
        main_window: WidgetUtils::get_widget_from_builder(builder, "main_window")?,
        settings_button: WidgetUtils::get_widget_from_builder(builder, "settings_button")?,
        sidebar_list_box: WidgetUtils::get_widget_from_builder(builder, "sidebar_list_box")?,
        create_profile_button: WidgetUtils::get_widget_from_builder(builder, "create_profile_button")?,
        multiplier_dropdown: WidgetUtils::get_widget_from_builder(builder, "multiplier_dropdown")?,
        flow_scale_entry: WidgetUtils::get_widget_from_builder(builder, "flow_scale_entry")?,
        performance_mode_switch: WidgetUtils::get_widget_from_builder(builder, "performance_mode_switch")?,
        hdr_mode_switch: WidgetUtils::get_widget_from_builder(builder, "hdr_mode_switch")?,
        experimental_present_mode_dropdown: WidgetUtils::get_widget_from_builder(builder, "experimental_present_mode_dropdown")?,
        main_settings_box: WidgetUtils::get_widget_from_builder(builder, "main_box")?,
    })
}

fn connect_signal_handlers(
    widgets: &AppWidgets,
    app_state: &Rc<RefCell<AppState>>,
    save_button: &gtk::Button,
) {
    // Connect settings button
    let main_window_clone = widgets.main_window.clone();
    let app_state_clone = app_state.clone();
    widgets.settings_button.connect_clicked(move |_| {
        let settings_win = settings_window::create_settings_window(&main_window_clone, app_state_clone.clone());
        settings_win.present();
    });

    // Connect profile-related signal handlers
    connect_profile_signal_handlers(app_state);
    connect_save_button_handler(save_button, app_state);
    connect_sidebar_row_activated_handler(&widgets.sidebar_list_box, app_state);
    connect_create_profile_button_handler(&widgets.create_profile_button, app_state);
}

fn initialize_ui(app_state: &Rc<RefCell<AppState>>) {
    let app_state_clone = app_state.clone();
    glib::idle_add_local(move || {
        let mut state = app_state_clone.borrow_mut();
        if state.config.game.first().is_some() {
            state.selected_profile_index = Some(0);
        }
        state.populate_sidebar_with_handlers(Some(app_state_clone.clone()));
        drop(state);
        
        if app_state_clone.borrow().selected_profile_index.is_some() {
            app_state_clone.borrow().update_main_window_from_profile();
        }
        glib::ControlFlow::Break
    });
}
