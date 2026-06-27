mod app;
mod keyboard;
mod input_handler;
mod key_sender;
mod profile;
mod ui;

use app::SpammyApp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), eframe::Error> {
    // Set up Ctrl+C handler with emergency key release
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();
    
    ctrlc::set_handler(move || {
        eprintln!("\n⚠️  CTRL+C DETECTED - Emergency shutdown!");
        
        // Force release all keys immediately
        if let Ok(sender_opt) = app::GLOBAL_KEY_SENDER.lock() {
            if let Some(sender) = sender_opt.as_ref() {
                let _ = sender.force_release_all();
            }
        }
        
        shutdown_flag_clone.store(true, Ordering::SeqCst);
        eprintln!("✓ All keys released. Exiting...");
        
        // Force exit after a short delay
        thread::sleep(std::time::Duration::from_millis(200));
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    let result = eframe::run_native(
        "Spammy",
        options,
        Box::new(|cc| Box::<SpammyApp>::new(SpammyApp::new(cc))),
    );
    
    // On window close, also release all keys
    if let Ok(sender_opt) = app::GLOBAL_KEY_SENDER.lock() {
        if let Some(sender) = sender_opt.as_ref() {
            let _ = sender.force_release_all();
        }
    }
    
    result
}
