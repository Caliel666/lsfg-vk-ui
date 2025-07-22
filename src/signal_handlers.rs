use gtk::prelude::*;
use gtk::{glib, DropDown, Entry, Switch, StringObject};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state::AppState;
use crate::utils::round_to_2_decimals;
use crate::profile_manager::update_profile_field;

/// Connects all profile-related signal handlers and stores their IDs
pub fn connect_profile_signal_handlers(app_state: &Rc<RefCell<AppState>>) {
    let mut state = app_state.borrow_mut();
    
    // Connect multiplier dropdown handler
    let multiplier_handler_id = connect_multiplier_dropdown_handler(
        &state.multiplier_dropdown,
        app_state,
    );
    state.multiplier_dropdown_handler_id = Some(multiplier_handler_id);
    
    // Connect flow scale entry handler
    let flow_handler_id = connect_flow_scale_entry_handler(
        &state.flow_scale_entry,
        app_state,
    );
    state.flow_scale_entry_handler_id = Some(flow_handler_id);
    
    // Connect performance mode switch handler
    let perf_handler_id = connect_performance_mode_switch_handler(
        &state.performance_mode_switch,
        app_state,
    );
    state.performance_mode_switch_handler_id = Some(perf_handler_id);
    
    // Connect HDR mode switch handler
    let hdr_handler_id = connect_hdr_mode_switch_handler(
        &state.hdr_mode_switch,
        app_state,
    );
    state.hdr_mode_switch_handler_id = Some(hdr_handler_id);
    
    // Connect experimental present mode dropdown handler
    let exp_handler_id = connect_experimental_present_mode_dropdown_handler(
        &state.experimental_present_mode_dropdown,
        app_state,
    );
    state.experimental_present_mode_dropdown_handler_id = Some(exp_handler_id);
}

/// Connects multiplier dropdown signal handler
fn connect_multiplier_dropdown_handler(
    dropdown: &DropDown,
    app_state: &Rc<RefCell<AppState>>,
) -> glib::SignalHandlerId {
    let app_state_clone = app_state.clone();
    dropdown.connect_selected_item_notify(move |dropdown| {
        update_profile_field(&app_state_clone, |profile| {
            if let Some(item) = dropdown.selected_item() {
                if let Some(string_obj) = item.downcast_ref::<StringObject>() {
                    let text = string_obj.string();
                    profile.multiplier = match text.as_str() {
                        "off" => 1,
                        _ => text.parse().unwrap_or(1),
                    };
                }
            }
            Ok(())
        });
    })
}

/// Connects flow scale entry signal handler
fn connect_flow_scale_entry_handler(
    entry: &Entry,
    app_state: &Rc<RefCell<AppState>>,
) -> glib::SignalHandlerId {
    let app_state_clone = app_state.clone();
    entry.connect_changed(move |entry| {
        update_profile_field(&app_state_clone, |profile| {
            if let Ok(value) = entry.text().parse::<f32>() {
                profile.flow_scale = round_to_2_decimals(value);
            }
            Ok(())
        });
    })
}

/// Connects performance mode switch signal handler
fn connect_performance_mode_switch_handler(
    switch: &Switch,
    app_state: &Rc<RefCell<AppState>>,
) -> glib::SignalHandlerId {
    let app_state_clone = app_state.clone();
    switch.connect_state_set(move |_sw, active| {
        update_profile_field(&app_state_clone, |profile| {
            profile.performance_mode = active;
            Ok(())
        });
        glib::Propagation::Proceed
    })
}

/// Connects HDR mode switch signal handler
fn connect_hdr_mode_switch_handler(
    switch: &Switch,
    app_state: &Rc<RefCell<AppState>>,
) -> glib::SignalHandlerId {
    let app_state_clone = app_state.clone();
    switch.connect_state_set(move |_sw, active| {
        update_profile_field(&app_state_clone, |profile| {
            profile.hdr_mode = active;
            Ok(())
        });
        glib::Propagation::Proceed
    })
}

/// Connects experimental present mode dropdown signal handler
fn connect_experimental_present_mode_dropdown_handler(
    dropdown: &DropDown,
    app_state: &Rc<RefCell<AppState>>,
) -> glib::SignalHandlerId {
    let app_state_clone = app_state.clone();
    dropdown.connect_selected_item_notify(move |dropdown| {
        update_profile_field(&app_state_clone, |profile| {
            let selected_text = dropdown.selected_item()
                .and_then(|item| item.downcast_ref::<StringObject>().map(|s| s.string().to_string()));
            if let Some(text) = selected_text {
                profile.experimental_present_mode = text;
            }
            Ok(())
        });
    })
}

/// Connects save button handler
pub fn connect_save_button_handler(
    save_button: &gtk::Button,
    app_state: &Rc<RefCell<AppState>>,
) {
    let app_state_clone = app_state.clone();
    save_button.connect_clicked(move |_| {
        handle_save_button_click(&app_state_clone);
    });
}

/// Handles save button click logic
fn handle_save_button_click(app_state: &Rc<RefCell<AppState>>) {
    let state_ref = app_state.borrow();
    if let Some(index) = state_ref.selected_profile_index {
        // Get current UI values
        let multiplier_str = state_ref.multiplier_dropdown.selected_item()
            .and_then(|item| item.downcast_ref::<StringObject>().map(|s| s.string().to_string()));
        let flow_scale_text = state_ref.flow_scale_entry.text().to_string();
        let performance_mode_active = state_ref.performance_mode_switch.is_active();
        let hdr_mode_active = state_ref.hdr_mode_switch.is_active();
        let exp_mode_str = state_ref.experimental_present_mode_dropdown.selected_item()
            .and_then(|item| item.downcast_ref::<StringObject>().map(|s| s.string().to_string()));
        
        let main_settings_box = state_ref.main_settings_box.clone();
        drop(state_ref);

        // Update profile with current values
        let mut state = app_state.borrow_mut();
        if let Some(profile) = state.config.game.get_mut(index) {
            if let Some(text) = multiplier_str {
                profile.multiplier = if text == "off" { 1 } else { text.parse().unwrap_or(1) };
            }

            if let Ok(value) = flow_scale_text.parse::<f32>() {
                profile.flow_scale = round_to_2_decimals(value);
            }

            profile.performance_mode = performance_mode_active;
            profile.hdr_mode = hdr_mode_active;

            if let Some(text) = exp_mode_str {
                profile.experimental_present_mode = text;
            }

            state.save_current_config();
            
            // Show feedback
            crate::ui_components::FeedbackUtils::show_temporary_feedback(&main_settings_box, "Saved!", 2);
        }
    }
}

/// Connects sidebar list box row activation handler
pub fn connect_sidebar_row_activated_handler(
    sidebar_list_box: &gtk::ListBox,
    app_state: &Rc<RefCell<AppState>>,
) {
    let app_state_clone = app_state.clone();
    sidebar_list_box.connect_row_activated(move |_list_box, row| {
        let index = row.index() as usize;
        let mut state = app_state_clone.borrow_mut();
        state.selected_profile_index = Some(index);
        drop(state);

        let app_state_for_idle = app_state_clone.clone();
        glib::idle_add_local(move || {
            app_state_for_idle.borrow().update_main_window_from_profile();
            glib::ControlFlow::Break
        });
    });
}

/// Connects create profile button handler
pub fn connect_create_profile_button_handler(
    create_profile_button: &gtk::Button,
    app_state: &Rc<RefCell<AppState>>,
) {
    let app_state_clone = app_state.clone();
    create_profile_button.connect_clicked(move |_| {
        crate::profile_manager::show_create_profile_dialog(app_state_clone.clone());
    });
}

/// Generic function to update dropdown selection by text value
pub fn update_dropdown_selection(dropdown: &DropDown, target_value: &str) -> bool {
    if let Some(model) = dropdown.model() {
        if let Some(list_model) = model.downcast_ref::<gtk::StringList>() {
            for i in 0..list_model.n_items() {
                if let Some(string_value) = list_model.string(i) {
                    if string_value.as_str() == target_value {
                        dropdown.set_selected(i);
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Blocks all profile-related signal handlers temporarily
pub fn with_blocked_signals<F, R>(app_state: &AppState, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Block signals
    let mult_guard = app_state.multiplier_dropdown_handler_id.as_ref()
        .map(|id| app_state.multiplier_dropdown.block_signal(id));
    let flow_guard = app_state.flow_scale_entry_handler_id.as_ref()
        .map(|id| app_state.flow_scale_entry.block_signal(id));
    let perf_guard = app_state.performance_mode_switch_handler_id.as_ref()
        .map(|id| app_state.performance_mode_switch.block_signal(id));
    let hdr_guard = app_state.hdr_mode_switch_handler_id.as_ref()
        .map(|id| app_state.hdr_mode_switch.block_signal(id));
    let exp_guard = app_state.experimental_present_mode_dropdown_handler_id.as_ref()
        .map(|id| app_state.experimental_present_mode_dropdown.block_signal(id));

    // Execute the function
    let result = f();

    // Signals are automatically unblocked when guards go out of scope
    let _ = mult_guard;
    let _ = flow_guard;
    let _ = perf_guard;
    let _ = hdr_guard;
    let _ = exp_guard;

    result
}