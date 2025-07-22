//! UI Components module
//! This module contains reusable UI component builders and factories

use gtk::prelude::*;
use gtk::{glib, MessageDialog, Entry, Button, Box, Orientation, ListBox, ListBoxRow, Label, ScrolledWindow};
use libadwaita::{ApplicationWindow, ActionRow, PreferencesGroup, Toast, ToastOverlay};
use libadwaita::prelude::*;

use crate::styles::css_classes;

/// Factory for creating standard message dialogs
#[allow(dead_code)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct DialogFactory;

impl DialogFactory {
    /// Creates a standard message dialog with consistent styling
    pub fn create_message_dialog(
        parent: &ApplicationWindow,
        title: &str,
        message: &str,
        message_type: gtk::MessageType,
    ) -> MessageDialog {
        let dialog = MessageDialog::new(
            Some(parent),
            gtk::DialogFlags::MODAL,
            message_type,
            gtk::ButtonsType::None,
            message,
        );
        dialog.set_title(Some(title));
        dialog
    }

    /// Creates a confirmation dialog with Cancel/Confirm buttons
    pub fn create_confirmation_dialog(
        parent: &ApplicationWindow,
        title: &str,
        message: &str,
        confirm_label: &str,
    ) -> MessageDialog {
        let dialog = Self::create_message_dialog(parent, title, message, gtk::MessageType::Warning);
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button(confirm_label, gtk::ResponseType::Other(1));
        dialog.set_default_response(gtk::ResponseType::Cancel);
        dialog
    }

    /// Creates an input dialog with an entry field
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_input_dialog(
        parent: &ApplicationWindow,
        title: &str,
        message: &str,
        placeholder: &str,
        initial_value: Option<&str>,
    ) -> (MessageDialog, Entry) {
        let dialog = Self::create_message_dialog(parent, title, message, gtk::MessageType::Question);
        
        let entry = Entry::builder()
            .placeholder_text(placeholder)
            .hexpand(true)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        if let Some(value) = initial_value {
            entry.set_text(value);
        }
        
        dialog.content_area().append(&entry);
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("OK", gtk::ResponseType::Other(1));
        dialog.set_default_response(gtk::ResponseType::Other(1));
        
        // Allow pressing Enter to trigger OK
        let dialog_clone = dialog.clone();
        entry.connect_activate(move |_| {
            dialog_clone.response(gtk::ResponseType::Other(1));
        });
        
        (dialog, entry)
    }

    /// Creates an input dialog with entry and browse button
    pub fn create_input_dialog_with_browse<F>(
        parent: &ApplicationWindow,
        title: &str,
        message: &str,
        placeholder: &str,
        initial_value: Option<&str>,
        browse_icon: &str,
        browse_tooltip: &str,
        browse_callback: F,
    ) -> (MessageDialog, Entry, Button)
    where
        F: Fn(&Entry) + 'static,
    {
        let dialog = Self::create_message_dialog(parent, title, message, gtk::MessageType::Question);
        
        let entry = Entry::builder()
            .placeholder_text(placeholder)
            .hexpand(true)
            .build();
        
        if let Some(value) = initial_value {
            entry.set_text(value);
        }
        
        let browse_button = Button::builder()
            .icon_name(browse_icon)
            .tooltip_text(browse_tooltip)
            .css_classes(css_classes::CIRCULAR_BUTTON)
            .build();
        
        let entry_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        entry_box.append(&entry);
        entry_box.append(&browse_button);
        dialog.content_area().append(&entry_box);
        
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("Create", gtk::ResponseType::Other(1));
        dialog.set_default_response(gtk::ResponseType::Other(1));
        
        // Connect browse button
        let entry_clone = entry.clone();
        browse_button.connect_clicked(move |_| {
            browse_callback(&entry_clone);
        });
        
        // Allow pressing Enter to trigger Create
        let dialog_clone = dialog.clone();
        let entry_clone = entry.clone();
        entry_clone.connect_activate(move |_| {
            dialog_clone.response(gtk::ResponseType::Other(1));
        });
        
        (dialog, entry, browse_button)
    }
}

/// Factory for creating process picker components
pub struct ProcessPickerFactory;

impl ProcessPickerFactory {
    /// Creates a process picker window
    pub fn create_process_picker_window(
        parent: &ApplicationWindow,
        processes: Vec<String>,
    ) -> (ApplicationWindow, ListBox) {
        let picker_window = ApplicationWindow::builder()
            .title("Select Process")
            .transient_for(parent)
            .modal(true)
            .default_width(400)
            .default_height(600)
            .css_classes(["process-picker-window"])
            .build();
        
        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .margin_top(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        let process_list_box = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .build();
        
        scrolled_window.set_child(Some(&process_list_box));
        
        let content_box = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        content_box.append(&scrolled_window);
        
        let close_button = Button::builder()
            .label("Close")
            .halign(gtk::Align::End)
            .margin_end(12)
            .margin_bottom(12)
            .build();
        content_box.append(&close_button);
        
        picker_window.set_content(Some(&content_box));
        
        // Populate the list with processes
        for proc_name in processes {
            let row = ListBoxRow::new();
            row.set_css_classes(css_classes::PROCESS_LIST_ITEM);
            
            let label = Label::builder()
                .label(&proc_name)
                .halign(gtk::Align::Start)
                .margin_start(12)
                .margin_end(12)
                .margin_top(8)
                .margin_bottom(8)
                .build();
            row.set_child(Some(&label));
            process_list_box.append(&row);
        }
        
        // Connect close button
        let picker_window_clone = picker_window.clone();
        close_button.connect_clicked(move |_| {
            picker_window_clone.close();
        });
        
        (picker_window, process_list_box)
    }
}

/// Factory for creating libadwaita preference components
#[allow(dead_code)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct PreferencesFactory;

impl PreferencesFactory {
    /// Creates a preferences group with title and description
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_preferences_group(title: &str, description: Option<&str>) -> PreferencesGroup {
        let mut builder = PreferencesGroup::builder().title(title);
        
        if let Some(desc) = description {
            builder = builder.description(desc);
        }
        
        builder.build()
    }

    /// Creates an action row with title and subtitle
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_action_row(title: &str, subtitle: Option<&str>) -> ActionRow {
        let mut builder = ActionRow::builder().title(title);
        
        if let Some(sub) = subtitle {
            builder = builder.subtitle(sub);
        }
        
        builder.build()
    }

    /// Creates an action row with an entry widget
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_entry_action_row(title: &str, placeholder: Option<&str>) -> (ActionRow, Entry) {
        let row = ActionRow::builder().title(title).build();
        
        let entry = Entry::builder()
            .hexpand(true)
            .build();
            
        if let Some(ph) = placeholder {
            entry.set_placeholder_text(Some(ph));
        }
        
        row.add_suffix(&entry);
        (row, entry)
    }
}

/// Factory for creating buttons with consistent styling
#[allow(dead_code)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct ButtonFactory;

impl ButtonFactory {
    /// Creates a circular icon button
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_circular_icon_button(icon_name: &str, tooltip: &str) -> Button {
        Button::builder()
            .icon_name(icon_name)
            .tooltip_text(tooltip)
            .css_classes(css_classes::CIRCULAR_BUTTON)
            .build()
    }

    /// Creates a destructive circular icon button
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_destructive_circular_button(icon_name: &str, tooltip: &str) -> Button {
        Button::builder()
            .icon_name(icon_name)
            .tooltip_text(tooltip)
            .css_classes(css_classes::DESTRUCTIVE_CIRCULAR_BUTTON)
            .build()
    }

    /// Creates a suggested action button
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_suggested_action_button(label: &str) -> Button {
        Button::builder()
            .label(label)
            .css_classes(css_classes::SUGGESTED_ACTION)
            .build()
    }

    /// Creates a flat icon button
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_flat_icon_button(icon_name: &str, tooltip: &str) -> Button {
        Button::builder()
            .icon_name(icon_name)
            .tooltip_text(tooltip)
            .css_classes(css_classes::ICON_BUTTON)
            .build()
    }
}

/// Factory for creating layout containers
#[allow(dead_code)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct LayoutFactory;

impl LayoutFactory {
    /// Creates a horizontal box with consistent spacing and margins
    pub fn create_horizontal_box_with_margins(spacing: i32) -> Box {
        Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(spacing)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .build()
    }

    /// Creates a vertical box with spacing
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_vertical_box(spacing: i32) -> Box {
        Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(spacing)
            .build()
    }

    /// Creates a centered box
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn create_centered_box(orientation: Orientation) -> Box {
        Box::builder()
            .orientation(orientation)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build()
    }
}

/// Utility functions for UI feedback
#[allow(dead_code)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct FeedbackUtils;

impl FeedbackUtils {
    /// Shows a temporary feedback message using a toast
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn show_toast(overlay: &ToastOverlay, message: &str, timeout: u32) {
        let toast = Toast::builder()
            .title(message)
            .timeout(timeout)
            .build();
        overlay.add_toast(toast);
    }

    /// Shows a success toast
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn show_success_toast(overlay: &ToastOverlay, message: &str) {
        let toast = Toast::builder()
            .title(message)
            .timeout(3)
            .build();
        overlay.add_toast(toast);
    }

    /// Shows an error toast
    #[allow(dead_code)]
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn show_error_toast(overlay: &ToastOverlay, message: &str) {
        let toast = Toast::builder()
            .title(message)
            .timeout(5)
            .build();
        overlay.add_toast(toast);
    }

    /// Shows a temporary feedback message in a container (legacy support)
    pub fn show_temporary_feedback(
        container: &Box,
        message: &str,
        duration_secs: u64,
    ) {
        let feedback_label = Label::new(Some(message));
        feedback_label.set_halign(gtk::Align::End);
        feedback_label.set_margin_end(12);
        feedback_label.set_margin_bottom(12);
        feedback_label.set_css_classes(&["feedback-label"]);
        
        container.append(&feedback_label);
        
        let container_clone = container.clone();
        let feedback_label_clone = feedback_label.clone();
        glib::timeout_add_local(std::time::Duration::new(duration_secs, 0), move || {
            container_clone.remove(&feedback_label_clone);
            glib::ControlFlow::Break
        });
    }
}

/// Widget utilities
pub struct WidgetUtils;

impl WidgetUtils {
    /// Creates a widget from builder with error handling
    pub fn get_widget_from_builder<T: glib::IsA<glib::Object>>(
        builder: &gtk::Builder,
        widget_id: &str,
    ) -> Result<T, String> {
        builder
            .object(widget_id)
            .ok_or_else(|| format!("Could not get {} from builder", widget_id))
    }
}