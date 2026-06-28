use egui::Color32;
use crate::app::SpammyApp;

pub fn draw_ui(ctx: &egui::Context, app: &mut SpammyApp) {
    // Light theme styling
    let mut style = ctx.style().as_ref().clone();
    style.visuals.dark_mode = false;
    style.visuals.panel_fill = Color32::from_rgb(240, 240, 240);
    style.visuals.extreme_bg_color = Color32::from_rgb(220, 220, 220);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(200, 200, 200);
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
    
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgb(240, 240, 240)).inner_margin(egui::Margin::same(12.0)))
        .show(ctx, |ui| {
            // Controls row
                    ui.horizontal(|ui| {
                        // Frequency control
                        ui.label("Frequency:");
                        let mut interval = app.get_repeat_interval_ms() as i32;
                        if ui.add(
                            egui::Slider::new(&mut interval, 10..=1000)
                                .step_by(10.0)
                                .show_value(true)
                                .text("ms")
                        ).changed() {
                            app.set_repeat_interval_ms(interval as u64);
                        }
                        
                        ui.separator();
                        
                        // Status
                        let enabled_text = if app.is_enabled() { "Enabled" } else { "Disabled" };
                        let enabled_color = if app.is_enabled() { Color32::from_rgb(0, 180, 0) } else { Color32::from_rgb(200, 0, 0) };
                        ui.colored_label(enabled_color, egui::RichText::new(enabled_text).strong());
                        
                        ui.separator();
                        
                        if ui.button("⚙ Toggle").clicked() {
                            app.enable(!app.is_enabled());
                        }
                    });
                    
                    ui.separator();
                    
                    // Profile controls
                    ui.horizontal(|ui| {
                        ui.label("Profile:");
                        
                        // Profile selector dropdown
                        let current_profile = app.get_active_profile_name().unwrap_or("Default").to_string();
                        let profile_names = app.get_profile_names();
                        
                        egui::ComboBox::from_id_source("profile_selector")
                            .selected_text(&current_profile)
                            .show_ui(ui, |ui| {
                                for name in &profile_names {
                                    if ui.selectable_label(name == &current_profile, name).clicked() {
                                        app.select_profile(name);
                                    }
                                }
                            });
                        
                        ui.separator();
                        
                        // Profile name input (for new or rename)
                        let mut new_name = app.get_new_profile_name().to_string();
                        ui.add(egui::TextEdit::singleline(&mut new_name)
                            .hint_text("Profile name")
                            .desired_width(120.0)
                            .text_color(Color32::BLACK));
                        app.set_new_profile_name(new_name.clone());
                        
                        // Save button - saves to text field name if provided, otherwise current profile
                        if ui.button("💾 Save").clicked() {
                            let save_name = if new_name.trim().is_empty() {
                                current_profile.clone()
                            } else {
                                new_name.trim().to_string()
                            };
                            app.save_current_as_profile(&save_name);
                            app.set_new_profile_name(String::new());
                        }
                        
                        // Delete button (only if more than 1 profile)
                        if profile_names.len() > 1 {
                            if ui.button("Del").clicked() {
                                app.delete_profile(&current_profile);
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    // Target window and input device controls
                    ui.horizontal(|ui| {
                        // Target window
                        if ui.button("📍 Target Window").clicked() {
                            app.toggle_window_picker();
                        }
                        
                        if let Some(name) = app.get_target_window_name() {
                            ui.label(format!("→ {}", name));
                        } else {
                            ui.colored_label(Color32::from_rgb(150, 150, 150), "All windows");
                        }
                        
                        if ui.button("✕").clicked() {
                            app.clear_target_window();
                        }
                        
                        ui.separator();
                        
                        // Input device selector
                        ui.label("Input:");
                        let current_device = app.get_current_input_device().unwrap_or("None").to_string();
                        let devices: Vec<_> = app.get_available_input_devices().to_vec();
                        
                        // Show just the device name, not full path
                        let current_display = devices.iter()
                            .find(|d| d.path == current_device)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| current_device.clone());
                        
                        egui::ComboBox::from_id_source("input_device_selector")
                            .selected_text(&current_display)
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for device in &devices {
                                    if ui.selectable_label(device.path == current_device, &device.name).clicked() {
                                        app.switch_input_device(&device.path);
                                    }
                                }
                            });
                    });
                    
                    // Window picker dropdown
                    if app.is_window_picker_open() {
                        ui.separator();
                        ui.label("Available Windows:");
                        
                        // Collect window list to avoid borrow issues
                        let windows: Vec<_> = app.get_available_windows().to_vec();
                        
                        ui.vertical(|ui| {
                            for (window_id, window_name) in windows {
                                if ui.button(format!("→ {}", window_name)).clicked() {
                                    app.set_target_window_by_id(window_id);
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                app.close_window_picker();
                            }
                        });
                        ui.separator();
                    }
                    
                    ui.separator();
                    
                    // Keyboard area
                    draw_keyboard(ui, app);
                    
                    ui.separator();
                    
                    // Stats
                    let active_count = app.get_active_keys().iter().filter(|&&k| k).count();
                    ui.horizontal(|ui| {
                        ui.label(format!("{} keys selected", active_count));
                    });
        });
    
    ctx.request_repaint();
}

fn draw_keyboard(ui: &mut egui::Ui, app: &mut SpammyApp) {
    let active_keys = app.get_active_keys().to_vec();
    let speedy_keys = app.get_speedy_keys().to_vec();
    let pressed_keys = app.get_pressed_keys().to_vec();
    let keyboard = app.get_keyboard_layout();
    let keys = keyboard.keys.clone();
    
    // Sizing - scaled up for better visibility
    let available_width = ui.available_width() - 5.0;
    let key_base_width = (available_width / 15.0).max(28.0);
    let key_height = 38.0;
    let font_size = 13.0;
    
    // Group keys by row
    let mut rows: Vec<Vec<_>> = vec![vec![]];
    for key in keys.iter() {
        while rows.len() <= key.row {
            rows.push(vec![]);
        }
        rows[key.row].push(key.clone());
    }
    
    // Draw each row with alignment padding
    for (_row_idx, row_keys) in rows.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
            
            // Calculate total width of this row
            let row_width: f32 = row_keys.iter().map(|k| k.width).sum();
            
            for key in row_keys {
                // Handle spacer keys - just add empty space
                if key.is_spacer {
                    let spacer_width = key_base_width * key.width;
                    ui.add_space(spacer_width);
                    continue;
                }
                
                let is_active = (key.code as usize) < active_keys.len() && active_keys[key.code as usize];
                let is_speedy = (key.code as usize) < speedy_keys.len() && speedy_keys[key.code as usize];
                let is_pressed = (key.code as usize) < pressed_keys.len() && pressed_keys[key.code as usize];
                
                // Determine key styling
                // Yellow = speedy mode, Orange = spammy mode, Gray = normal
                let bg_color = if is_speedy && is_pressed {
                    Color32::from_rgb(220, 180, 0)  // Darker yellow when speedy AND pressed
                } else if is_speedy {
                    Color32::from_rgb(255, 220, 0)  // Yellow for Speedy Mode
                } else if is_active && is_pressed {
                    Color32::from_rgb(200, 100, 0)  // Darker orange when active AND pressed
                } else if is_active {
                    Color32::from_rgb(255, 140, 0)  // Orange for Spammy (active)
                } else if is_pressed {
                    Color32::from_rgb(100, 100, 100)  // Darker gray when physically pressed
                } else {
                    Color32::from_rgb(200, 200, 200)  // Light gray inactive
                };
                
                // Use black text for yellow/light backgrounds, white for dark gray
                let text_color = if is_speedy || !is_pressed {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                };
                
                let key_width = key_base_width * key.width;
                
                // Create button without outline
                let response = ui.add_sized(
                    egui::vec2(key_width, key_height),
                    egui::Button::new(
                        egui::RichText::new(&key.name)
                            .color(text_color)
                            .size(font_size)
                            .strong()
                    )
                    .fill(bg_color)
                    .stroke(egui::Stroke::new(0.0, Color32::TRANSPARENT))
                );
                
                // Left click = toggle spammy mode
                if response.clicked() {
                    app.toggle_key(key.code as usize);
                }
                
                // Right click = toggle speedy mode
                if response.secondary_clicked() {
                    app.toggle_speedy_key(key.code as usize);
                }
            }
            
            // Add padding at the end to align all rows
            // Max row width is 14 units (number row), so pad shorter rows
            let max_row_width = 14.0;
            let padding_width = (max_row_width - row_width) * key_base_width;
            if padding_width > 0.0 {
                ui.add_space(padding_width);
            }
        });
    }
    
    // Legend
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("■ Yellow = Speedy").color(Color32::from_rgb(180, 140, 0)).strong().size(14.0));
        ui.label(egui::RichText::new("?").color(Color32::from_rgb(60, 60, 60)).size(14.0).strong())
            .on_hover_text("Right-click a key to enable Speedy mode.\nSends a single quick tap when you press the key.");
        ui.separator();
        ui.label(egui::RichText::new("■ Orange = Spammy").color(Color32::from_rgb(255, 140, 0)).strong().size(14.0));
        ui.label(egui::RichText::new("?").color(Color32::from_rgb(60, 60, 60)).size(14.0).strong())
            .on_hover_text("Left-click a key to enable Spammy mode.\nRepeats the key while you hold it down.");
        ui.separator();
        ui.label(egui::RichText::new("■ Gray = Normal").color(Color32::from_rgb(80, 80, 80)).strong().size(14.0));
    });
}
