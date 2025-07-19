// src/main.rs
use gtk::prelude::*;
use gtk::{glib, CssProvider, Builder, MessageDialog, Label}; // Added Label for feedback
use libadwaita::ApplicationWindow;
// Removed: use libadwaita::NavigationPage; // This type is no longer directly used
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, io};
use std::path::PathBuf;
use toml;
use dirs;

// --- Configuration Data Structures ---

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    version: u32,
    #[serde(default)]
    pub game: Vec<GameProfile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameProfile {
    pub exe: String,
    pub multiplier: u32,
    #[serde(serialize_with = "serialize_flow_scale", deserialize_with = "deserialize_flow_scale")]
    pub flow_scale: f32,
    pub performance_mode: bool,
    pub hdr_mode: bool,
    pub experimental_present_mode: String,
}

// Default values for a new game profile
impl Default for GameProfile {
    fn default() -> Self {
        GameProfile {
            exe: String::new(),
            multiplier: 1, // Default to "off" (1)
            flow_scale: round_to_2_decimals(0.7),
            performance_mode: true,
            hdr_mode: false,
            experimental_present_mode: "vsync".to_string(),
        }
    }
}


// --- Application State ---

struct AppState {
    config: Config,
    selected_profile_index: Option<usize>,
    // Store references to the UI widgets for easy access and updates
    main_window: ApplicationWindow,
    sidebar_list_box: gtk::ListBox,
    multiplier_dropdown: gtk::DropDown,
    flow_scale_entry: gtk::Entry,
    performance_mode_switch: gtk::Switch,
    hdr_mode_switch: gtk::Switch,
    experimental_present_mode_dropdown: gtk::DropDown,
    save_button: gtk::Button, // Added save button reference
    main_settings_box: gtk::Box, // Reference to the main content GtkBox (your 'main_box')
    main_stack: gtk::Stack, // Reference to the GtkStack
    // Store SignalHandlerIds to block/unblock signals
    multiplier_dropdown_handler_id: Option<glib::SignalHandlerId>,
    flow_scale_entry_handler_id: Option<glib::SignalHandlerId>,
    performance_mode_switch_handler_id: Option<glib::SignalHandlerId>,
    hdr_mode_switch_handler_id: Option<glib::SignalHandlerId>,
    experimental_present_mode_dropdown_handler_id: Option<glib::SignalHandlerId>,
}

impl AppState {
    // Saves the current configuration to the TOML file
    fn save_current_config(&self) {
        if let Err(e) = save_config(&self.config) {
            eprintln!("Failed to save config: {}", e);
            // In a real app, you'd show a user-friendly error dialog here
        }
    }

    // Updates the main window UI with data from the currently selected profile
    fn update_main_window_from_profile(&self) {
        if let Some(index) = self.selected_profile_index {
            if let Some(profile) = self.config.game.get(index) {
                // Temporarily block signals to prevent re-entrancy
                let _guard_mult = self.multiplier_dropdown_handler_id.as_ref().map(|id| self.multiplier_dropdown.block_signal(id));
                let _guard_flow = self.flow_scale_entry_handler_id.as_ref().map(|id| self.flow_scale_entry.block_signal(id));
                let _guard_perf = self.performance_mode_switch_handler_id.as_ref().map(|id| self.performance_mode_switch.block_signal(id));
                let _guard_hdr = self.hdr_mode_switch_handler_id.as_ref().map(|id| self.hdr_mode_switch.block_signal(id));
                let _guard_exp = self.experimental_present_mode_dropdown_handler_id.as_ref().map(|id| self.experimental_present_mode_dropdown.block_signal(id));

                // Update Multiplier Dropdown
                let multiplier_str = match profile.multiplier {
                    1 => "off".to_string(),
                    _ => profile.multiplier.to_string(),
                };
                if let Some(pos) = self.multiplier_dropdown.model().and_then(|model| {
                    let list_model = model.downcast_ref::<gtk::StringList>()?;
                    // Compare GString with &str by converting GString to &str
                    (0..list_model.n_items()).find(|&i| list_model.string(i).map_or(false, |s| s.as_str() == multiplier_str))
                }) {
                    self.multiplier_dropdown.set_selected(pos);
                }

                // Update Flow Scale Entry (round to avoid floating point display issues)
                let rounded_flow_scale = round_to_2_decimals(profile.flow_scale);
                self.flow_scale_entry.set_text(&format!("{:.2}", rounded_flow_scale));

                // Update Performance Mode Switch
                self.performance_mode_switch.set_active(profile.performance_mode);

                // Update HDR Mode Switch
                self.hdr_mode_switch.set_active(profile.hdr_mode);

                // Update Experimental Present Mode Dropdown
                if let Some(pos) = self.experimental_present_mode_dropdown.model().and_then(|model| {
                    let list_model = model.downcast_ref::<gtk::StringList>()?;
                    // Compare GString with &str by converting GString to &str
                    (0..list_model.n_items()).find(|&i| list_model.string(i).map_or(false, |s| s.as_str() == profile.experimental_present_mode))
                }) {
                    self.experimental_present_mode_dropdown.set_selected(pos);
                }
                // Signal handlers are unblocked automatically when _guard_X go out of scope

                // Switch to the settings page
                self.main_stack.set_visible_child_name("settings_page");

            }
        } else {
            // Clear or disable main window elements if no profile is selected
            self.multiplier_dropdown.set_selected(0); // Default to 'off' or first item
            self.flow_scale_entry.set_text("");
            self.performance_mode_switch.set_active(false);
            self.hdr_mode_switch.set_active(false);
            self.experimental_present_mode_dropdown.set_selected(0); // Default to first item

            // Switch to the about page
            self.main_stack.set_visible_child_name("about_page");
        }
    }



    // Populates sidebar with optional app_state for button handlers
    fn populate_sidebar_with_handlers(&self, app_state: Option<Rc<RefCell<AppState>>>) {
        // Clear existing rows
        while let Some(child) = self.sidebar_list_box.first_child() {
            self.sidebar_list_box.remove(&child);
        }

        for (i, profile) in self.config.game.iter().enumerate() {
            let row = gtk::ListBoxRow::new();
            
            // Create a horizontal box to hold the profile name and buttons
            let row_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .margin_start(12)
                .margin_end(12)
                .margin_top(8)
                .margin_bottom(8)
                .build();

            // Profile name label
            let label = gtk::Label::builder()
                .label(&profile.exe)
                .halign(gtk::Align::Start)
                .hexpand(true)
                .build();

            // Edit button
            let edit_button = gtk::Button::builder()
                .label("🖊")
                .css_classes(["flat", "circular"])
                .tooltip_text("Edit profile name")
                .build();

            // Remove button
            let remove_button = gtk::Button::builder()
                .label("✕")
                .css_classes(["flat", "circular", "destructive-action"])
                .tooltip_text("Remove profile")
                .build();

            // Add all elements to the row box
            row_box.append(&label);
            row_box.append(&edit_button);
            row_box.append(&remove_button);

            // Connect button handlers if app_state is available
            if let Some(app_state_ref) = &app_state {
                // Edit button handler
                let app_state_clone = app_state_ref.clone();
                let profile_index = i;
                edit_button.connect_clicked(move |_| {
                    let state = app_state_clone.borrow();
                    let main_window = &state.main_window;
                    
                    // Create edit dialog
                    let dialog = MessageDialog::new(
                        Some(main_window),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Question,
                        gtk::ButtonsType::None,
                        "Edit profile name:",
                    );
                    dialog.set_title(Some("Edit Profile"));

                    let entry = gtk::Entry::builder()
                        .placeholder_text("Profile Name")
                        .text(&state.config.game[profile_index].exe)
                        .margin_top(12)
                        .margin_bottom(12)
                        .margin_start(12)
                        .margin_end(12)
                        .build();
                    
                    dialog.content_area().append(&entry);
                    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
                    dialog.add_button("Save", gtk::ResponseType::Other(1));
                    dialog.set_default_response(gtk::ResponseType::Other(1));

                    let app_state_clone_dialog = app_state_clone.clone();
                    let entry_clone = entry.clone();
                    dialog.connect_response(move |d, response| {
                        if response == gtk::ResponseType::Other(1) {
                            let new_name = entry_clone.text().to_string();
                            if !new_name.is_empty() {
                                let mut state = app_state_clone_dialog.borrow_mut();
                                
                                // Check if profile with this name already exists (excluding current)
                                if state.config.game.iter().enumerate().any(|(idx, p)| idx != profile_index && p.exe == new_name) {
                                    let error_dialog = MessageDialog::new(
                                        Some(d),
                                        gtk::DialogFlags::MODAL,
                                        gtk::MessageType::Error,
                                        gtk::ButtonsType::Ok,
                                        "A profile with this name already exists",
                                    );
                                    error_dialog.set_title(Some("Error"));
                                    error_dialog.connect_response(move |d, _| { d.close(); });
                                    error_dialog.present();
                                    return;
                                }

                                // Update profile name
                                state.config.game[profile_index].exe = new_name;
                                state.save_current_config();
                                state.populate_sidebar_with_handlers(Some(app_state_clone_dialog.clone()));
                            }
                        }
                        d.close();
                    });
                    dialog.present();
                });

                // Remove button handler
                let app_state_clone = app_state_ref.clone();
                let profile_index = i;
                remove_button.connect_clicked(move |_| {
                    let state = app_state_clone.borrow();
                    let main_window = &state.main_window;
                    let profile_name = &state.config.game[profile_index].exe;
                    
                    // Create confirmation dialog
                    let dialog = MessageDialog::new(
                        Some(main_window),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Warning,
                        gtk::ButtonsType::None,
                        &format!("Are you sure you want to remove the profile '{}'?", profile_name),
                    );
                    dialog.set_title(Some("Remove Profile"));
                    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
                    dialog.add_button("Remove", gtk::ResponseType::Other(1));
                    dialog.set_default_response(gtk::ResponseType::Cancel);

                    let app_state_clone_dialog = app_state_clone.clone();
                    dialog.connect_response(move |d, response| {
                        if response == gtk::ResponseType::Other(1) {
                            let mut state = app_state_clone_dialog.borrow_mut();
                            
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
                            state.populate_sidebar_with_handlers(Some(app_state_clone_dialog.clone()));
                            drop(state);
                            
                            // Update main window
                            app_state_clone_dialog.borrow().update_main_window_from_profile();
                        }
                        d.close();
                    });
                    dialog.present();
                });
            }

            row.set_child(Some(&row_box));
            self.sidebar_list_box.append(&row);

            // Set the selected state
            if self.selected_profile_index == Some(i) {
                self.sidebar_list_box.select_row(Some(&row));
            }
        }
    }
}

// --- Utility Functions ---

fn round_to_2_decimals(value: f32) -> f32 {
    // Use string formatting to get exactly 2 decimal places and then parse back
    // This avoids floating point precision issues
    format!("{:.2}", value).parse().unwrap_or(value)
}

// Custom serde functions to ensure flow_scale is always rounded
fn serialize_flow_scale<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Force to 2 decimal places and serialize as a precise decimal
    let rounded = round_to_2_decimals(*value);
    let formatted = format!("{:.2}", rounded);
    let precise_value: f64 = formatted.parse().unwrap_or(*value as f64);
    serializer.serialize_f64(precise_value)
}

fn deserialize_flow_scale<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let value = f64::deserialize(deserializer)?;
    Ok(round_to_2_decimals(value as f32))
}

// --- Configuration File Handling Functions ---

fn get_config_path() -> Result<PathBuf, io::Error> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find config directory"))?
        .join("lsfg-vk");
    
    fs::create_dir_all(&config_dir)?; // Ensure directory exists
    println!("Config directory: {:?}", config_dir);
    Ok(config_dir.join("conf.toml"))
}


fn load_config() -> Result<Config, io::Error> {
    let config_path = get_config_path()?;
    println!("Attempting to load config from: {:?}", config_path);
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path)?;
        println!("Successfully read config contents ({} bytes).", contents.len());
        let mut config: Config = toml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse TOML: {}", e)))?;
        
        // Clean up any floating point precision issues in existing configs
        let mut needs_save = false;
        for profile in &mut config.game {
            let original = profile.flow_scale;
            profile.flow_scale = round_to_2_decimals(profile.flow_scale);
            if (original - profile.flow_scale).abs() > f32::EPSILON {
                needs_save = true;
            }
        }
        
        // Save the cleaned config if we made changes
        if needs_save {
            let _ = save_config(&config);
        }
        
        Ok(config)
    } else {
        println!("Config file not found at {:?}, creating default.", config_path);
        Ok(Config { version: 1, game: Vec::new() })
    }
}

fn save_config(config: &Config) -> Result<(), io::Error> {
    let config_path = get_config_path()?;
    println!("Attempting to save config to: {:?}", config_path);
    let toml_string = toml::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to serialize TOML: {}", e)))?;
    fs::write(&config_path, toml_string)?;
    println!("Successfully saved config.");
    Ok(())
}

// --- Main Application Logic ---

fn main() -> glib::ExitCode {
    let application = libadwaita::Application::builder()
        .application_id("com.cali666.lsfg-vk-ui")
        .build();
    
    // Set the desktop file name for proper GNOME integration
    glib::set_application_name("LSFG-VK UI");
    glib::set_prgname(Some("lsfg-vk-ui"));

    application.connect_startup(move |_app| { // Renamed app to _app
        // Load CSS for sidebar background
let provider = CssProvider::new();
provider.load_from_data(&format!(
    ".sidebar {{
        background-color: @theme_bg_color;
    }}
    
    .sidebar-content {{
        background-color: shade(@theme_bg_color, {});
        color: @theme_fg_color;
        padding: 12px;
    }}",
    0.95  // Slightly lighter/darker shade if needed
));
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Set up icon theme for the application icon
        if let Some(display) = gtk::gdk::Display::default() {
            let icon_theme = gtk::IconTheme::for_display(&display);
            icon_theme.add_resource_path("/com/cali666/lsfg-vk-ui/icons");
        }
    });

    application.connect_activate(move |app| {
        // Load initial configuration
        let initial_config = load_config().unwrap_or_else(|e| { // Removed mut
            eprintln!("Error loading config: {}", e);
            Config { version: 1, game: Vec::new() }
        });

        // Load UI from .ui file
        let ui_bytes = include_bytes!("../resources/ui.ui");
        let builder = Builder::from_string(std::str::from_utf8(ui_bytes).unwrap());

        // Get main window and other widgets
        let main_window: ApplicationWindow = builder // Use libadwaita::ApplicationWindow
            .object("main_window")
            .expect("Could not get main_window from builder");
        main_window.set_application(Some(app));

        // Set application icon for proper dock integration
        main_window.set_icon_name(Some("com.cali666.lsfg-vk-ui"));

        let sidebar_list_box: gtk::ListBox = builder
            .object("sidebar_list_box")
            .expect("Could not get sidebar_list_box from builder");
        let create_profile_button: gtk::Button = builder
            .object("create_profile_button")
            .expect("Could not get create_profile_button from builder");

        let multiplier_dropdown: gtk::DropDown = builder
            .object("multiplier_dropdown")
            .expect("Could not get multiplier_dropdown from builder");
        let flow_scale_entry: gtk::Entry = builder
            .object("flow_scale_entry")
            .expect("Could not get flow_scale_entry from builder");
        let performance_mode_switch: gtk::Switch = builder
            .object("performance_mode_switch")
            .expect("Could not get performance_mode_switch from builder");
        let hdr_mode_switch: gtk::Switch = builder
            .object("hdr_mode_switch")
            .expect("Could not get hdr_mode_switch from builder");
        let experimental_present_mode_dropdown: gtk::DropDown = builder
            .object("experimental_present_mode_dropdown")
            .expect("Could not get experimental_present_mode_dropdown from builder");

        // Get the GtkStack and GtkStackSwitcher
        let main_stack: gtk::Stack = builder
            .object("main_stack")
            .expect("Could not get main_stack from builder. Ensure it has id='main_stack' in ui.ui.");
        let main_stack_switcher: gtk::StackSwitcher = builder
            .object("main_stack_switcher")
            .expect("Could not get main_stack_switcher from builder. Ensure it has id='main_stack_switcher' in ui.ui.");
        
        // Connect the switcher to the stack
        main_stack_switcher.set_stack(Some(&main_stack));


        // Get the GtkBox that contains the settings (your 'main_box' from ui.ui)
        let main_settings_box: gtk::Box = builder
            .object("main_box")
            .expect("Could not get main_box from builder");


        // Create the Save button
        let save_button = gtk::Button::builder()
            .label("Save Changes")
            .halign(gtk::Align::End) // Align to the end (right)
            .margin_end(12)
            .margin_bottom(12)
            .build();

        // Append the save button to the main_settings_box
        main_settings_box.append(&save_button);


        // Initialize application state (with None for handler IDs initially)
        let app_state = Rc::new(RefCell::new(AppState {
            config: initial_config, // initial_config is now in scope
            selected_profile_index: None,
            main_window: main_window.clone(),
            sidebar_list_box: sidebar_list_box.clone(),
            multiplier_dropdown: multiplier_dropdown.clone(),
            flow_scale_entry: flow_scale_entry.clone(),
            performance_mode_switch: performance_mode_switch.clone(),
            hdr_mode_switch: hdr_mode_switch.clone(),
            experimental_present_mode_dropdown: experimental_present_mode_dropdown.clone(),
            save_button: save_button.clone(), // Store reference to save button
            main_settings_box: main_settings_box.clone(), // Store reference to main_settings_box
            main_stack: main_stack.clone(), // Store reference to main_stack
            multiplier_dropdown_handler_id: None,
            flow_scale_entry_handler_id: None,
            performance_mode_switch_handler_id: None,
            hdr_mode_switch_handler_id: None,
            experimental_present_mode_dropdown_handler_id: None,
        }));

        // --- Connect Signals ---

        // Handle profile selection in sidebar
        let app_state_clone = app_state.clone();
        sidebar_list_box.connect_row_activated(move |_list_box, row| {
            let index = row.index() as usize; // Cast to usize
            let mut state = app_state_clone.borrow_mut();
            state.selected_profile_index = Some(index);
            drop(state); // Explicitly drop the mutable borrow

            // Defer the UI update to avoid RefCell re-entrancy panic
            let app_state_for_idle = app_state_clone.clone();
            glib::idle_add_local(move || {
                app_state_for_idle.borrow().update_main_window_from_profile();
                glib::ControlFlow::Break // Run once
            });
        });

        // Handle "Create New Profile" button click
        let app_state_clone = app_state.clone();
        create_profile_button.connect_clicked(move |_| {
            // Create the dialog using gtk::MessageDialog
            let dialog = MessageDialog::new(
                Some(&app_state_clone.borrow().main_window), // Parent window
                gtk::DialogFlags::MODAL, // Flags
                gtk::MessageType::Question, // Message type
                gtk::ButtonsType::None, // No default buttons, we'll add custom ones
                "Enter the name for the new game profile:", // Message body
            );
            dialog.set_title(Some("Create New Game Profile")); // Set title explicitly, wrapped in Some()

            let entry = gtk::Entry::builder()
                .placeholder_text("Game Name")
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(12)
                .margin_end(12)
                .build();
            
            // Add the entry to the dialog's content area
            dialog.content_area().append(&entry);

            // Add custom responses
            dialog.add_button("Cancel", gtk::ResponseType::Cancel);
            dialog.add_button("Create", gtk::ResponseType::Other(1)); // Use 1 for "create" response

            dialog.set_default_response(gtk::ResponseType::Other(1)); // Set default response

            let app_state_clone_dialog = app_state_clone.clone();
            let entry_clone = entry.clone(); // Clone entry for the closure
            dialog.connect_response(
                move |d: &MessageDialog, response: gtk::ResponseType| { // Use gtk::ResponseType
                    if response == gtk::ResponseType::Other(1) { // Compare with ResponseType::Other(1)
                        let game_name = entry_clone.text().to_string(); // Use cloned entry
                        if !game_name.is_empty() {
                            let mut state = app_state_clone_dialog.borrow_mut();
                            
                            // Check if profile already exists
                            if state.config.game.iter().any(|p| p.exe == game_name) {
                                let error_dialog = MessageDialog::new(
                                    Some(d), // Parent is the current dialog
                                    gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Error,
                                    gtk::ButtonsType::Ok,
                                    "A profile with this name already exists",
                                );
                                error_dialog.set_title(Some("Error")); // Wrapped in Some()
                                error_dialog.connect_response(move |d, _| { d.close(); });
                                error_dialog.present();
                                return;
                            }

                            // Create new profile with defaults
                            let new_profile = GameProfile {
                                exe: game_name,
                                ..Default::default()
                            };
                            
                            state.config.game.push(new_profile);
                            state.selected_profile_index = Some(state.config.game.len() - 1);
                            
                            // Save and update UI
                            if let Err(e) = save_config(&state.config) {
                                eprintln!("Failed to save config: {}", e);
                            }
                            
                            state.populate_sidebar_with_handlers(Some(app_state_clone_dialog.clone()));
                            drop(state); // Explicitly drop the mutable borrow

                            // Defer the UI update to avoid potential re-entrancy during initial setup
                            let app_state_for_idle = app_state_clone_dialog.clone();
                            glib::idle_add_local(move || {
                                app_state_for_idle.borrow().update_main_window_from_profile();
                                glib::ControlFlow::Break // Run once
                            });
                        }
                    }
                    d.close();
                }
            );
            dialog.present();
        });

        // Connect signals for main window controls to update the selected profile (in-memory only)
        let app_state_clone_for_handler_mult = app_state.clone();
        let multiplier_handler_id = multiplier_dropdown.connect_selected_item_notify(move |dropdown| {
            let mut state = app_state_clone_for_handler_mult.borrow_mut();
            
            if let Some(index) = state.selected_profile_index {
                if index < state.config.game.len() { // Safety check
                    if let Some(profile) = state.config.game.get_mut(index) {
                        if let Some(item) = dropdown.selected_item() {
                            if let Some(string_obj) = item.downcast_ref::<gtk::StringObject>() {
                                let text = string_obj.string();
                                profile.multiplier = match text.as_str() {
                                    "off" => 1,
                                    _ => text.parse().unwrap_or(1),
                                };
                                // Removed save_config here
                            }
                        }
                    }
                }
            }
        });

        app_state.borrow_mut().multiplier_dropdown_handler_id = Some(multiplier_handler_id);


        let app_state_clone_for_handler_flow = app_state.clone();
        let flow_handler_id = flow_scale_entry.connect_changed(move |entry| {
            let mut state = app_state_clone_for_handler_flow.borrow_mut(); // Use the cloned app_state
            if let Some(index) = state.selected_profile_index {
                if let Some(profile) = state.config.game.get_mut(index) {
                    if let Ok(value) = entry.text().parse::<f32>() {
                        profile.flow_scale = round_to_2_decimals(value);
                        // Removed save_config here
                    }
                }
            }
        });
        app_state.borrow_mut().flow_scale_entry_handler_id = Some(flow_handler_id);


        let app_state_clone_for_handler_perf = app_state.clone();
        let perf_handler_id = performance_mode_switch.connect_state_set(move |_sw, active| { // Renamed sw to _sw
            let mut state = app_state_clone_for_handler_perf.borrow_mut(); // Use the cloned app_state
            if let Some(index) = state.selected_profile_index {
                if let Some(profile) = state.config.game.get_mut(index) {
                    profile.performance_mode = active;
                    // Removed save_config here
                }
            }
            drop(state); // Explicitly drop the mutable borrow
            glib::Propagation::Proceed
        });
        app_state.borrow_mut().performance_mode_switch_handler_id = Some(perf_handler_id);


        let app_state_clone_for_handler_hdr = app_state.clone();
        let hdr_handler_id = hdr_mode_switch.connect_state_set(move |_sw, active| { // Renamed sw to _sw
            let mut state = app_state_clone_for_handler_hdr.borrow_mut(); // Use the cloned app_state
            if let Some(index) = state.selected_profile_index {
                if let Some(profile) = state.config.game.get_mut(index) {
                    profile.hdr_mode = active;
                    // Removed save_config here
                }
            }
            drop(state); // Explicitly drop the mutable borrow

            // Defer the UI update to avoid RefCell re-entrancy panic
            let app_state_for_idle = app_state_clone_for_handler_hdr.clone();
            glib::idle_add_local(move || {
                app_state_for_idle.borrow().update_main_window_from_profile();
                glib::ControlFlow::Break // Run once
            });
            glib::Propagation::Proceed
        });
        app_state.borrow_mut().hdr_mode_switch_handler_id = Some(hdr_handler_id);


        let app_state_clone_for_handler_exp = app_state.clone();
        let exp_handler_id = experimental_present_mode_dropdown.connect_selected_item_notify(move |dropdown| {
            let mut state = app_state_clone_for_handler_exp.borrow_mut(); // Use the cloned app_state
            if let Some(index) = state.selected_profile_index {
                if let Some(profile) = state.config.game.get_mut(index) {
                    let selected_text = dropdown.selected_item().and_then(|item| item.downcast_ref::<gtk::StringObject>().map(|s| s.string().to_string()));
                    if let Some(text) = selected_text {
                        profile.experimental_present_mode = text;
                        // Removed save_config here
                    }
                }
            }
        });
        app_state.borrow_mut().experimental_present_mode_dropdown_handler_id = Some(exp_handler_id);

        // Connect the Save Changes button
        let app_state_clone_save = app_state.clone();
        save_button.connect_clicked(move |_| {
            let state_ref = app_state_clone_save.borrow(); // Immutable borrow to read UI values
            if let Some(index) = state_ref.selected_profile_index {
                // Read values from UI elements
                let multiplier_str = state_ref.multiplier_dropdown.selected_item().and_then(|item| item.downcast_ref::<gtk::StringObject>().map(|s| s.string().to_string()));
                let flow_scale_text = state_ref.flow_scale_entry.text().to_string();
                let performance_mode_active = state_ref.performance_mode_switch.is_active();
                let hdr_mode_active = state_ref.hdr_mode_switch.is_active();
                let exp_mode_str = state_ref.experimental_present_mode_dropdown.selected_item().and_then(|item| item.downcast_ref::<gtk::StringObject>().map(|s| s.string().to_string()));
                
                drop(state_ref); // Drop immutable borrow before mutable borrow

                let mut state = app_state_clone_save.borrow_mut(); // Mutable borrow to update profile
                if let Some(profile) = state.config.game.get_mut(index) {
                    // Update the profile with read values
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

                    // Provide visual feedback
                    let feedback_label = Label::new(Some("Saved!"));
                    feedback_label.set_halign(gtk::Align::End);
                    feedback_label.set_margin_end(12);
                    feedback_label.set_margin_bottom(12);
                    
                    let main_settings_box_clone = state.main_settings_box.clone(); // Clone main_settings_box for the timeout closure
                    
                    // Temporarily add the feedback label to the main_settings_box
                    main_settings_box_clone.append(&feedback_label);


                    glib::timeout_add_local(std::time::Duration::new(2, 0), move || {
                        // Remove feedback label
                        main_settings_box_clone.remove(&feedback_label);
                        glib::ControlFlow::Break
                    });
                }
            }
        });


        // Initial population and UI update, ensure it's deferred and safe
        let app_state_clone_initial = app_state.clone();
        glib::idle_add_local(move || {
            let mut state = app_state_clone_initial.borrow_mut();
            // Select the first profile if available, otherwise clear main window
            if state.config.game.first().is_some() {
                state.selected_profile_index = Some(0);
            }
            state.populate_sidebar_with_handlers(Some(app_state_clone_initial.clone())); // Populate after setting selection
            drop(state); // Drop the mutable borrow
            
            if app_state_clone_initial.borrow().selected_profile_index.is_some() {
                app_state_clone_initial.borrow().update_main_window_from_profile();
            }
            glib::ControlFlow::Break
        });


        main_window.present();
    });

    application.run()
}
