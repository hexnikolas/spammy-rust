use egui::Color32;
use crate::app::SpammyApp;

pub fn draw_ui(ctx: &egui::Context, app: &mut SpammyApp) {
    // Custom styling - compact
    let mut style = ctx.style().as_ref().clone();
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = Color32::from_rgb(30, 30, 30);
    style.visuals.extreme_bg_color = Color32::from_rgb(20, 20, 20);
    style.spacing.item_spacing = egui::vec2(4.0, 3.0);
    style.spacing.button_padding = egui::vec2(4.0, 2.0);
    ctx.set_style(style);
    
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgb(25, 25, 25)).inner_margin(egui::Margin::same(8.0)))
        .show(ctx, |ui| {
            // Header
            ui.vertical(|ui| {
                ui.heading(egui::RichText::new("SPAMMY").color(Color32::from_rgb(255, 165, 0)).size(20.0));
                
                // Status bar
                ui.horizontal(|ui| {
                    if ui.button("⚙ Toggle Spamming").clicked() {
                        app.enable(!app.is_enabled());
                    }
                    
                    let enabled_text = if app.is_enabled() { "● ENABLED" } else { "● DISABLED" };
                    let enabled_color = if app.is_enabled() { Color32::GREEN } else { Color32::RED };
                    ui.colored_label(enabled_color, egui::RichText::new(enabled_text).size(12.0));
                });
                
                ui.separator();
                
                // Repeat interval control
                ui.horizontal(|ui| {
                    ui.label("Repeat Speed (ms):");
                    let mut interval = app.get_repeat_interval_ms() as i32;
                    if ui.add(
                        egui::Slider::new(&mut interval, 10..=1000)
                            .step_by(10.0)
                            .show_value(true)
                    ).changed() {
                        app.set_repeat_interval_ms(interval as u64);
                    }
                });
                
                // Target window control
                ui.horizontal(|ui| {
                    if ui.button("📍 Select Target Window").clicked() {
                        app.toggle_window_picker();
                    }
                    
                    if let Some(name) = app.get_target_window_name() {
                        ui.label(format!("Target: {}", name));
                    } else {
                        ui.colored_label(Color32::from_rgb(200, 100, 100), "No target (all windows)");
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
                draw_keyboard(ui, app);
                
                ui.separator();
                
                // Stats
                let active_count = app.get_active_keys().iter().filter(|&&k| k).count();
                let pressed_count = app.get_pressed_keys().iter().filter(|&&k| k).count();
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Active: {}", active_count)).color(Color32::from_rgb(255, 165, 0)).size(11.0));
                    ui.separator();
                    ui.label(egui::RichText::new(format!("Pressed: {}", pressed_count)).color(Color32::from_rgb(100, 200, 255)).size(11.0));
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
    
    // Compact sizing
    let available_width = ui.available_width() - 5.0;
    let key_base_width = (available_width / 13.5).max(15.0);
    let key_height = 20.0;
    let font_size = 9.0;
    
    // Group keys by row
    let mut rows: Vec<Vec<_>> = vec![vec![]];
    for key in keys.iter() {
        while rows.len() <= key.row {
            rows.push(vec![]);
        }
        rows[key.row].push(key.clone());
    }
    
    // Draw each row
    for row_keys in rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
            
            for key in row_keys {
                let is_active = (key.code as usize) < active_keys.len() && active_keys[key.code as usize];
                let is_pressed = (key.code as usize) < pressed_keys.len() && pressed_keys[key.code as usize];
                
                // Determine key styling
                let bg_color = if is_pressed {
                    Color32::from_rgb(60, 60, 60)  // Lighter gray when pressed
                } else if is_active {
                    Color32::from_rgb(255, 165, 0)  // Orange when active
                } else {
                    Color32::from_rgb(70, 70, 70)  // Gray inactive
                };
                
                let text_color = if is_active {
                    Color32::from_rgb(0, 0, 0)  // Black text on orange
                } else {
                    Color32::from_rgb(220, 220, 220)  // Light gray text
                };
                
                let key_width = key_base_width * key.width;
                
                // Create button with styling
                let response = ui.add_sized(
                    egui::vec2(key_width, key_height),
                    egui::Button::new(
                        egui::RichText::new(&key.name)
                            .color(text_color)
                            .size(font_size)
                    )
                    .fill(bg_color)
                    .stroke(egui::Stroke::new(0.5, Color32::from_rgb(50, 50, 50)))
                );
                
                if response.clicked() {
                    app.toggle_key(key.code as usize);
                }
            }
        });
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
    }
}
