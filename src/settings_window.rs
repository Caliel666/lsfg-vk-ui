use gtk::prelude::*;
use gtk::{glib, Switch, Button};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow, PreferencesGroup, PreferencesPage, PreferencesWindow, ActionRow, Toast, ToastOverlay, HeaderBar, WindowTitle};
use std::rc::Rc;
use std::cell::RefCell;

use crate::app_state::AppState;

pub fn create_settings_window(parent: &ApplicationWindow, app_state: Rc<RefCell<AppState>>) -> PreferencesWindow {
    let settings_window = PreferencesWindow::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .search_enabled(false)
        .default_width(600)
        .default_height(500)
        .build();

    // Create custom header bar
    let header_bar = HeaderBar::builder()
        .show_end_title_buttons(true)
        .build();
    
    let title_widget = WindowTitle::builder()
        .title("Settings")
        .build();
    header_bar.set_title_widget(Some(&title_widget));
    
    // Add save button to header bar
    let save_button = Button::builder()
        .label("Save")
        .css_classes(["suggested-action"])
        .build();
    header_bar.pack_end(&save_button);
    
    // Create toast overlay for feedback
    let toast_overlay = ToastOverlay::new();

    // Create Global Settings page
    let global_page = create_global_settings_page(app_state.clone(), &toast_overlay, &save_button);
    settings_window.add(&global_page);
    
    // Create About page
    let about_page = create_about_page();
    settings_window.add(&about_page);

    settings_window
}

fn create_global_settings_page(app_state: Rc<RefCell<AppState>>, toast_overlay: &ToastOverlay, save_button: &Button) -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Global Settings")
        .icon_name("preferences-system-symbolic")
        .build();

    let group = PreferencesGroup::builder()
        .title("Global Settings")
        .description("Configure global application settings")
        .build();

    // --- Custom DLL Toggle and Path using modern libadwaita components ---
    let custom_dll_switch = Switch::builder()
        .halign(gtk::Align::End)
        .valign(gtk::Align::Center)
        .build();

    let custom_dll_row = ActionRow::builder()
        .title("Use Custom Lossless.dll")
        .subtitle("Enable to specify a custom path to the Lossless.dll file")
        .build();
    custom_dll_row.add_suffix(&custom_dll_switch);
    custom_dll_row.set_activatable_widget(Some(&custom_dll_switch));

    let custom_dll_path_row = ActionRow::builder()
        .title("DLL Path")
        .subtitle("Path to the custom Lossless.dll file")
        .sensitive(false)
        .build();
    
    let custom_dll_entry = gtk::Entry::builder()
        .placeholder_text("/path/to/Lossless.dll")
        .hexpand(true)
        .sensitive(false)
        .build();
    
    custom_dll_path_row.add_suffix(&custom_dll_entry);

    group.add(&custom_dll_row);
    group.add(&custom_dll_path_row);

    // Initial state setup for Custom DLL
    let current_dll_path = app_state.borrow().config.ordered_global.global.as_ref()
        .and_then(|g| g.dll.clone());

    if let Some(path) = current_dll_path {
        custom_dll_switch.set_active(true);
        custom_dll_entry.set_text(&path);
        custom_dll_path_row.set_sensitive(true);
        custom_dll_entry.set_sensitive(true);
    } else {
        custom_dll_switch.set_active(false);
        custom_dll_path_row.set_sensitive(false);
        custom_dll_entry.set_sensitive(false);
    }

    // Connect switch to enable/disable entry and update config
    let app_state_clone_switch = app_state.clone();
    let custom_dll_path_row_clone = custom_dll_path_row.clone();
    let custom_dll_entry_clone = custom_dll_entry.clone();
    custom_dll_switch.connect_state_set(move |_sw, active| {
        custom_dll_path_row_clone.set_sensitive(active);
        custom_dll_entry_clone.set_sensitive(active);
        let mut state = app_state_clone_switch.borrow_mut();
        if active {
            // If activating, ensure global config exists and set DLL path
            let current_path = custom_dll_entry_clone.text().to_string();
            state.config.ordered_global.global.get_or_insert_with(Default::default).dll = Some(current_path);
        } else {
            // If deactivating, set DLL path to None
            if let Some(global_config) = state.config.ordered_global.global.as_mut() {
                global_config.dll = None;
            }
        }
        glib::Propagation::Proceed
    });

    // Connect entry to update config
    let app_state_clone_entry = app_state.clone();
    let custom_dll_switch_clone = custom_dll_switch.clone();
    custom_dll_entry.connect_changed(move |entry| {
        let mut state = app_state_clone_entry.borrow_mut();
        if custom_dll_switch_clone.is_active() {
            let path = entry.text().to_string();
            if !path.is_empty() {
                state.config.ordered_global.global.get_or_insert_with(Default::default).dll = Some(path);
            } else {
                // If path is cleared, set dll to None
                if let Some(global_config) = state.config.ordered_global.global.as_mut() {
                    global_config.dll = None;
                }
            }
        }
    });

    // Connect save button
    let app_state_clone_save = app_state.clone();
    let toast_overlay_clone = toast_overlay.clone();
    save_button.connect_clicked(move |_| {
        let state = app_state_clone_save.borrow();
        state.save_current_config();
        
        // Show toast notification
        let toast = Toast::builder()
            .title("Settings saved successfully")
            .timeout(2)
            .build();
        toast_overlay_clone.add_toast(toast);
    });
    
    page.add(&group);
    page
}

fn create_about_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("About")
        .icon_name("help-about-symbolic")
        .build();

    // Links group
    let links_group = PreferencesGroup::builder()
        .title("Useful Links")
        .description("External resources and information")
        .build();

    // LSFG-VK Wiki link
    let wiki_row = ActionRow::builder()
        .title("LSFG-VK Wiki")
        .subtitle("Documentation and guides")
        .build();
    
    let wiki_button = Button::builder()
        .icon_name("adw-external-link-symbolic")
        .tooltip_text("Open LSFG-VK Wiki")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();
    
    wiki_button.connect_clicked(|_| {
        let _ = gtk::gio::AppInfo::launch_default_for_uri(
            "https://github.com/PancakeTAS/lsfg-vk/wiki",
            gtk::gio::AppLaunchContext::NONE,
        );
    });
    
    wiki_row.add_suffix(&wiki_button);
    links_group.add(&wiki_row);

    // Buy Lossless Scaling link
    let buy_row = ActionRow::builder()
        .title("Buy Lossless Scaling")
        .subtitle("Purchase the official Lossless Scaling software")
        .build();
    
    let buy_button = Button::builder()
        .icon_name("adw-external-link-symbolic")
        .tooltip_text("Open Steam Store")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();
    
    buy_button.connect_clicked(|_| {
        let _ = gtk::gio::AppInfo::launch_default_for_uri(
            "https://store.steampowered.com/app/993090/Lossless_Scaling/",
            gtk::gio::AppLaunchContext::NONE,
        );
    });
    
    buy_row.add_suffix(&buy_button);
    links_group.add(&buy_row);

    // Discord link
    let discord_row = ActionRow::builder()
        .title("Lossless Scaling Discord")
        .subtitle("Join the community discussion")
        .build();
    
    let discord_button = Button::builder()
        .icon_name("adw-external-link-symbolic")
        .tooltip_text("Join Discord Server")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();
    
    discord_button.connect_clicked(|_| {
        let _ = gtk::gio::AppInfo::launch_default_for_uri(
            "https://discord.gg/losslessscaling",
            gtk::gio::AppLaunchContext::NONE,
        );
    });
    
    discord_row.add_suffix(&discord_button);
    links_group.add(&discord_row);
    page.add(&links_group);
    page
}
