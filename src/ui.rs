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
            // Make scrollable area for controls to prevent cutoff
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
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
                    
                    // Target window control
                    ui.horizontal(|ui| {
                        if ui.button("📍 Target Window").clicked() {
                            app.toggle_window_picker();
                        }
                        
                        if let Some(name) = app.get_target_window_name() {
                            ui.label(format!("→ {}", name));
                        } else {
                            ui.colored_label(Color32::from_rgb(200, 0, 0), "No target");
                        }
                        
                        if ui.button("✕").clicked() {
                            app.clear_target_window();
                        }
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
        });
    
    ctx.request_repaint();
}

fn draw_keyboard(ui: &mut egui::Ui, app: &mut SpammyApp) {
    let active_keys = app.get_active_keys().to_vec();
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
    for (row_idx, row_keys) in rows.iter().enumerate() {
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
                let is_pressed = (key.code as usize) < pressed_keys.len() && pressed_keys[key.code as usize];
                
                // Determine key styling - match reference image colors
                let bg_color = if is_active && is_pressed {
                    Color32::from_rgb(200, 100, 0)  // Darker orange when active AND pressed
                } else if is_active {
                    Color32::from_rgb(255, 140, 0)  // Orange for Spammy (active)
                } else if is_pressed {
                    Color32::from_rgb(100, 100, 100)  // Darker gray when physically pressed
                } else {
                    Color32::from_rgb(200, 200, 200)  // Light gray inactive (like reference)
                };
                
                let text_color = if is_active {
                    Color32::BLACK  // Black text on orange background
                } else {
                    Color32::BLACK  // Black text on light gray
                };
                
                let key_width = key_base_width * key.width;
                
                // Create button with better borders for keyboard feel
                let response = ui.add_sized(
                    egui::vec2(key_width, key_height),
                    egui::Button::new(
                        egui::RichText::new(&key.name)
                            .color(text_color)
                            .size(font_size)
                            .strong()
                    )
                    .fill(bg_color)
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80)))
                );
                
                if response.clicked() {
                    app.toggle_key(key.code as usize);
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
        ui.colored_label(Color32::from_rgb(200, 40, 40), "■ Red = Disabled");
        ui.separator();
        ui.colored_label(Color32::from_rgb(255, 140, 0), "■ Orange = Spammy");
        ui.separator();
        ui.colored_label(Color32::from_rgb(200, 200, 200), "■ Gray = Ready");
    });
}
