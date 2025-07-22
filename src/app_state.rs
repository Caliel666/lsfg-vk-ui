use gtk::prelude::*;
use gtk::{glib, ListBoxRow, Label, Button};
use libadwaita::ApplicationWindow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Config, save_config};
use crate::utils::round_to_2_decimals;
use crate::signal_handlers::{with_blocked_signals, update_dropdown_selection};
use crate::ui_components::LayoutFactory;
use crate::styles::css_classes;
use crate::profile_manager::{show_edit_profile_dialog, show_remove_profile_dialog};

#[allow(dead_code)]
pub struct AppState {
    pub config: Config,
    pub selected_profile_index: Option<usize>,
    // Store references to the UI widgets for easy access and updates
    pub main_window: ApplicationWindow,
    pub sidebar_list_box: gtk::ListBox,
    pub multiplier_dropdown: gtk::DropDown,
    pub flow_scale_entry: gtk::Entry,
    pub performance_mode_switch: gtk::Switch,
    pub hdr_mode_switch: gtk::Switch,
    pub experimental_present_mode_dropdown: gtk::DropDown,
    pub save_button: gtk::Button,
    pub main_settings_box: gtk::Box,
    // Store SignalHandlerIds to block/unblock signals
    pub multiplier_dropdown_handler_id: Option<glib::SignalHandlerId>,
    pub flow_scale_entry_handler_id: Option<glib::SignalHandlerId>,
    pub performance_mode_switch_handler_id: Option<glib::SignalHandlerId>,
    pub hdr_mode_switch_handler_id: Option<glib::SignalHandlerId>,
    pub experimental_present_mode_dropdown_handler_id: Option<glib::SignalHandlerId>,
}

impl AppState {
    // Saves the current configuration to the TOML file
    pub fn save_current_config(&self) {
        if let Err(e) = save_config(&self.config) {
            eprintln!("Failed to save config: {}", e);
            // In a real app, you'd show a user-friendly error dialog here
        }
    }

    // Updates the main window UI with data from the currently selected profile
    pub fn update_main_window_from_profile(&self) {
        if let Some(index) = self.selected_profile_index {
            if let Some(profile) = self.config.game.get(index) {
                let profile_clone = profile.clone();
                let multiplier_dropdown = self.multiplier_dropdown.clone();
                let flow_scale_entry = self.flow_scale_entry.clone();
                let performance_mode_switch = self.performance_mode_switch.clone();
                let hdr_mode_switch = self.hdr_mode_switch.clone();
                let experimental_present_mode_dropdown = self.experimental_present_mode_dropdown.clone();

                // Temporarily block signals to prevent re-entrancy
                with_blocked_signals(self, || {
                    // Update Multiplier Dropdown
                    let multiplier_str = match profile_clone.multiplier {
                        1 => "off",
                        _ => &profile_clone.multiplier.to_string(),
                    };
                    update_dropdown_selection(&multiplier_dropdown, multiplier_str);

                    // Update Flow Scale Entry (round to avoid floating point display issues)
                    let rounded_flow_scale = round_to_2_decimals(profile_clone.flow_scale);
                    flow_scale_entry.set_text(&format!("{:.2}", rounded_flow_scale));

                    // Update Performance Mode Switch
                    performance_mode_switch.set_active(profile_clone.performance_mode);

                    // Update HDR Mode Switch
                    hdr_mode_switch.set_active(profile_clone.hdr_mode);

                    // Update Experimental Present Mode Dropdown
                    update_dropdown_selection(&experimental_present_mode_dropdown, &profile_clone.experimental_present_mode);
                });
            }
        } else {
            self.clear_main_window_ui();
        }
    }

    // Clears the main window UI when no profile is selected
    fn clear_main_window_ui(&self) {
        self.multiplier_dropdown.set_selected(0);
        self.flow_scale_entry.set_text("");
        self.performance_mode_switch.set_active(false);
        self.hdr_mode_switch.set_active(false);
        self.experimental_present_mode_dropdown.set_selected(0);
    }

    // Populates sidebar with optional app_state for button handlers
    pub fn populate_sidebar_with_handlers(&self, app_state: Option<Rc<RefCell<AppState>>>) {
        // Clear existing rows
        while let Some(child) = self.sidebar_list_box.first_child() {
            self.sidebar_list_box.remove(&child);
        }

        let mut row_to_select: Option<ListBoxRow> = None;

        for (i, profile) in self.config.game.iter().enumerate() {
            let row = self.create_profile_row(profile, i, &app_state);
            self.sidebar_list_box.append(&row);

            // Mark the row to be selected later
            if self.selected_profile_index == Some(i) {
                row_to_select = Some(row.clone());
            }
        }

        // Perform selection in a separate idle callback
        if let Some(row) = row_to_select {
            let list_box_clone = self.sidebar_list_box.clone();
            glib::idle_add_local(move || {
                list_box_clone.select_row(Some(&row));
                glib::ControlFlow::Break
            });
        }
    }

    // Creates a single profile row with edit and remove buttons
    fn create_profile_row(
        &self,
        profile: &crate::config::GameProfile,
        index: usize,
        app_state: &Option<Rc<RefCell<AppState>>>,
    ) -> ListBoxRow {
        let row = ListBoxRow::new();
        let row_box = LayoutFactory::create_horizontal_box_with_margins(8);

        // Profile name label
        let label = Label::builder()
            .label(&profile.exe)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();

        // Edit button
        let edit_button = Button::builder()
            .icon_name("document-edit-symbolic")
            .css_classes(css_classes::CIRCULAR_BUTTON)
            .tooltip_text("Edit profile name")
            .build();

        // Remove button
        let remove_button = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(css_classes::DESTRUCTIVE_CIRCULAR_BUTTON)
            .tooltip_text("Remove profile")
            .build();

        row_box.append(&label);
        row_box.append(&edit_button);
        row_box.append(&remove_button);

        // Connect button handlers if app_state is available
        if let Some(app_state_ref) = app_state {
            let app_state_clone = app_state_ref.clone();
            edit_button.connect_clicked(move |_| {
                show_edit_profile_dialog(app_state_clone.clone(), index);
            });

            let app_state_clone = app_state_ref.clone();
            remove_button.connect_clicked(move |_| {
                show_remove_profile_dialog(app_state_clone.clone(), index);
            });
        }

        row.set_child(Some(&row_box));
        row
    }
}
