use gtk::prelude::*;
use gtk::{glib, Entry};
use libadwaita::ApplicationWindow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Config, GameProfile};
use crate::app_state::AppState;
use crate::ui_components::{DialogFactory, ProcessPickerFactory};
use crate::utils::get_vulkan_processes;

/// Validates if a profile name is unique and not empty
pub fn validate_profile_name(config: &Config, name: &str, exclude_index: Option<usize>) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    
    let name = name.trim();
    let exists = config.game.iter().enumerate().any(|(idx, profile)| {
        profile.exe == name && exclude_index.map_or(true, |exclude_idx| idx != exclude_idx)
    });
    
    if exists {
        return Err("A profile with this name already exists".to_string());
    }
    
    Ok(())
}

/// Shows an error dialog for profile validation failures
pub fn show_profile_error<W: gtk::prelude::IsA<gtk::Window>>(parent: &W, error_message: &str) {
    let error_dialog = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        error_message,
    );
    error_dialog.set_title(Some("Error"));
    error_dialog.connect_response(move |d, _| { d.close(); });
    error_dialog.present();
}

/// Creates a new profile dialog and handles the creation process
pub fn show_create_profile_dialog(app_state: Rc<RefCell<AppState>>) {
    let main_window = app_state.borrow().main_window.clone();
    
    let main_window_clone = main_window.clone();
    let (dialog, entry, _browse_button) = DialogFactory::create_input_dialog_with_browse(
        &main_window,
        "New Profile",
        "Enter or browse Application Name",
        "Application Name",
        None,
        "application-x-executable-symbolic",
        "Pick a running Vulkan process",
        move |entry: &Entry| {
            show_process_picker_with_parent(entry, &main_window_clone);
        },
    );
    
    let app_state_clone = app_state.clone();
    let entry_clone = entry.clone();
    dialog.connect_response(move |d, response| {
        if response == gtk::ResponseType::Other(1) {
            let profile_name = entry_clone.text().to_string();
            match handle_create_profile(&app_state_clone, &profile_name) {
                Ok(_) => d.close(),
                Err(error_msg) => {
                    show_profile_error(d, &error_msg);
                }
            }
        } else {
            d.close();
        }
    });
    
    dialog.present();
}

/// Handles the actual profile creation logic
fn handle_create_profile(app_state: &Rc<RefCell<AppState>>, profile_name: &str) -> Result<(), String> {
    let mut state = app_state.borrow_mut();
    
    // Validate profile name
    validate_profile_name(&state.config, profile_name, None)?;
    
    // Create new profile
    let new_profile = GameProfile {
        exe: profile_name.to_string(),
        ..Default::default()
    };
    
    state.config.game.push(new_profile);
    state.selected_profile_index = Some(state.config.game.len() - 1);
    
    state.save_current_config();
    state.populate_sidebar_with_handlers(Some(app_state.clone()));
    drop(state);
    
    // Update UI in idle callback
    let app_state_clone = app_state.clone();
    glib::idle_add_local(move || {
        app_state_clone.borrow().update_main_window_from_profile();
        glib::ControlFlow::Break
    });
    
    Ok(())
}

/// Shows the process picker dialog
fn show_process_picker(entry: &Entry) {
    let processes = get_vulkan_processes();
    
    // Try to get the main window, but handle the case where it fails
    let main_window = match entry.root().and_then(|root| root.downcast::<ApplicationWindow>().ok()) {
        Some(window) => window,
        None => {
            eprintln!("Could not get main window from entry root");
            return;
        }
    };
    
    let (picker_window, process_list_box) = ProcessPickerFactory::create_process_picker_window(&main_window, processes);
    
    // Connect selection handler
    let entry_clone = entry.clone();
    let picker_window_clone = picker_window.clone();
    process_list_box.connect_row_activated(move |_list_box, row| {
        if let Some(label_widget) = row.child().and_then(|c| c.downcast::<gtk::Label>().ok()) {
            let process_info = label_widget.label().to_string();
            // Extract just the process name from "PID 1234: process_name" format
            if let Some(colon_pos) = process_info.find(": ") {
                let process_name = &process_info[colon_pos + 2..];
                entry_clone.set_text(process_name);
            } else {
                // Fallback to full text if format is unexpected
                entry_clone.set_text(&process_info);
            }
            picker_window_clone.close();
        }
    });
    
    picker_window.present();
}

/// Shows the process picker dialog with a provided parent window
fn show_process_picker_with_parent(entry: &Entry, parent: &ApplicationWindow) {
    let processes = get_vulkan_processes();
    let (picker_window, process_list_box) = ProcessPickerFactory::create_process_picker_window(parent, processes);
    
    // Connect selection handler
    let entry_clone = entry.clone();
    let picker_window_clone = picker_window.clone();
    process_list_box.connect_row_activated(move |_list_box, row| {
        if let Some(label_widget) = row.child().and_then(|c| c.downcast::<gtk::Label>().ok()) {
            let process_info = label_widget.label().to_string();
            // Extract just the process name from "PID 1234: process_name" format
            if let Some(colon_pos) = process_info.find(": ") {
                let process_name = &process_info[colon_pos + 2..];
                entry_clone.set_text(process_name);
            } else {
                // Fallback to full text if format is unexpected
                entry_clone.set_text(&process_info);
            }
            picker_window_clone.close();
        }
    });
    
    picker_window.present();
}

/// Shows the edit profile dialog
pub fn show_edit_profile_dialog(app_state: Rc<RefCell<AppState>>, profile_index: usize) {
    let state = app_state.borrow();
    let main_window = state.main_window.clone();
    let current_name = state.config.game[profile_index].exe.clone();
    drop(state);
    
    let main_window_clone = main_window.clone();
    let (dialog, entry, _browse_button) = DialogFactory::create_input_dialog_with_browse(
        &main_window,
        "Edit Profile",
        "Edit profile name or browse for a process:",
        "Profile Name",
        Some(&current_name),
        "application-x-executable-symbolic",
        "Pick a running Vulkan process",
        move |entry: &Entry| {
            show_process_picker_with_parent(entry, &main_window_clone);
        },
    );
    
    let app_state_clone = app_state.clone();
    let entry_clone = entry.clone();
    dialog.connect_response(move |d, response| {
        if response == gtk::ResponseType::Other(1) {
            let new_name = entry_clone.text().to_string();
            match handle_edit_profile(&app_state_clone, profile_index, &new_name) {
                Ok(_) => d.close(),
                Err(error_msg) => {
                    show_profile_error(d, &error_msg);
                }
            }
        } else {
            d.close();
        }
    });
    
    dialog.present();
}

/// Handles the actual profile editing logic
fn handle_edit_profile(app_state: &Rc<RefCell<AppState>>, profile_index: usize, new_name: &str) -> Result<(), String> {
    let mut state = app_state.borrow_mut();
    
    // Validate profile name
    validate_profile_name(&state.config, new_name, Some(profile_index))?;
    
    // Update profile name
    state.config.game[profile_index].exe = new_name.to_string();
    state.save_current_config();
    state.populate_sidebar_with_handlers(Some(app_state.clone()));
    
    Ok(())
}

/// Shows the remove profile confirmation dialog
pub fn show_remove_profile_dialog(app_state: Rc<RefCell<AppState>>, profile_index: usize) {
    let state = app_state.borrow();
    let main_window = state.main_window.clone();
    let profile_name = state.config.game[profile_index].exe.clone();
    drop(state);
    
    let dialog = DialogFactory::create_confirmation_dialog(
        &main_window,
        "Remove Profile",
        &format!("Are you sure you want to remove the profile '{}'?", profile_name),
        "Remove",
    );
    
    let app_state_clone = app_state.clone();
    dialog.connect_response(move |d, response| {
        if response == gtk::ResponseType::Other(1) {
            handle_remove_profile(&app_state_clone, profile_index);
        }
        d.close();
    });
    
    dialog.present();
}

/// Handles the actual profile removal logic
fn handle_remove_profile(app_state: &Rc<RefCell<AppState>>, profile_index: usize) {
    let mut state = app_state.borrow_mut();
    
    // Remove the profile
    state.config.game.remove(profile_index);
    
    // Update selected index if needed
    if let Some(selected) = state.selected_profile_index {
        if selected == profile_index {
            // If we removed the selected profile, select the first available or none
            state.selected_profile_index = if state.config.game.is_empty() { None } else { Some(0) };
        } else if selected > profile_index {
            // Adjust index if we removed a profile before the selected one
            state.selected_profile_index = Some(selected - 1);
        }
    }
    
    state.save_current_config();
    state.populate_sidebar_with_handlers(Some(app_state.clone()));
    drop(state);
    
    // Update main window
    app_state.borrow().update_main_window_from_profile();
}

/// Updates a profile field with validation
pub fn update_profile_field<F>(
    app_state: &Rc<RefCell<AppState>>,
    field_updater: F,
) where
    F: FnOnce(&mut GameProfile) -> Result<(), String>,
{
    let mut state = app_state.borrow_mut();
    if let Some(index) = state.selected_profile_index {
        if let Some(profile) = state.config.game.get_mut(index) {
            if let Err(e) = field_updater(profile) {
                eprintln!("Error updating profile field: {}", e);
            }
        }
    }
}