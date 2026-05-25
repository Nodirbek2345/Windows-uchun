mod config;
mod state;
mod detector;
mod filter;
mod proxy;
mod gui;
mod mitm;
mod logger;

use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use config::Config;
use state::AppState;
use detector::Detector;
use filter::MappingStore;
use proxy::ProxyServer;
use eframe::egui;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem},
    Icon,
};

/// 32x32 shield icon RGBA — programmatik yaratish (fayl kerak emas)
fn create_tray_icon() -> Icon {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let cx = x as f32 - 16.0;
            let cy = y as f32 - 16.0;
            let dist = (cx * cx + cy * cy).sqrt();

            // Shield shape: circle
            if dist < 14.0 {
                // Purple gradient
                let r = 100 + ((dist / 14.0) * 30.0) as u8;
                let g = 60 + ((1.0 - dist / 14.0) * 40.0) as u8;
                let b = 220;
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("Tray icon yaratib bo'lmadi")
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    logger::init_logger();

    println!("Starting PrivacyProxy...");

    // 1. Load config
    let config = Config::load("config.yaml").unwrap_or_else(|_| {
        println!("config.yaml topilmadi, standart sozlamalar ishlatilmoqda.");
        Config {
            proxy: config::ProxyConfig { port: 8081, host: "127.0.0.1".into(), ssl_insecure: false },
            target_sites: vec!["chat.openai.com".into()],
            text_filter: config::TextFilterConfig {
                enabled: true,
                methods: config::MethodsConfig { regex: true, blacklist: false },
                detect: config::DetectConfig {
                    names: true, phone: true, email: true, card_number: true, address: true,
                    organization: true, passport: true, pinfl: true, date_of_birth: false
                },
                mask_style: "label".into(),
                blacklist: vec![],
            },
            image_filter: config::ImageFilterConfig {
                enabled: true,
                face: config::FaceConfig { enabled: true, method: "blur".into(), blur_radius: 25 },
                ocr: config::OcrConfig { enabled: false, languages: vec![] },
                metadata: config::MetadataConfig { strip_gps: true, strip_device: true, strip_datetime: true },
            },
            mapping: config::MappingConfig { storage: "memory".into(), ttl_minutes: 30, encrypt: false },
            logging: config::LoggingConfig { enabled: true, level: "INFO".into(), max_entries: 1000, save_to_file: false },
            ui: config::UiConfig { language: "uz".into(), theme: "dark".into(), start_minimized: false, autostart: false }
        }
    });

    let start_minimized = config.ui.start_minimized;

    // 2. Init state
    let state = Arc::new(AppState::new(config.clone()));
    
    // 3. Init Detector and Mapping Store
    let detector = Arc::new(Detector::new());
    let mapping = MappingStore::new();

    // 4. Start Proxy Server in background
    let proxy_state = state.clone();
    let proxy_target_sites = config.target_sites.clone();
    tokio::spawn(async move {
        let server = ProxyServer::new(
            config.proxy.port,
            proxy_state,
            detector,
            mapping,
            proxy_target_sites,
        );
        if let Err(e) = server.run().await {
            eprintln!("Proxy server stopped: {}", e);
        }
    });

    // 5. Tray icon context menu
    let menu = Menu::new();
    let item_show = MenuItem::new("🔓 Ko'rsatish", true, None);
    let item_quit = MenuItem::new("❌ Chiqish", true, None);
    let _ = menu.append(&item_show);
    let _ = menu.append(&item_quit);
    
    let show_id = item_show.id().clone();
    let quit_id = item_quit.id().clone();

    // Tray icon reference (Windows/macOS: must be on main thread after event loop starts)
    let tray_icon_holder: Rc<RefCell<Option<tray_icon::TrayIcon>>> = Rc::new(RefCell::new(None));
    let tray_c = tray_icon_holder.clone();

    let icon = create_tray_icon();

    // 6. Start GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_title("PrivacyProxy (O'zbek)"),
        ..Default::default()
    };
    
    let state_for_cleanup = state.clone();

    let res = eframe::run_native(
        "PrivacyProxy (O'zbek)",
        options,
        Box::new(move |cc| {
            // Create tray icon inside the event loop context (Windows requirement)
            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("PrivacyProxy — fonda ishlamoqda")
                .with_icon(icon)
                .build()
                .expect("Tray icon yaratib bo'lmadi");
            
            tray_c.borrow_mut().replace(tray);
            
            Ok(Box::new(gui::PrivacyProxyApp::new(
                cc,
                state.clone(),
                show_id.clone(),
                quit_id.clone(),
                start_minimized,
            )))
        }),
    );
    
    // Cleanup proxy immediately on app termination
    state_for_cleanup.sync_system_proxy(false);
    println!("Dastur to'xtatildi, Windows system proxy o'chirildi.");
    res
}
