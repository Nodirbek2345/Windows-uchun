#![windows_subsystem = "windows"]
mod config;
mod state;
mod detector;
mod filter;
mod proxy;
mod gui;
mod mitm;
mod logger;
mod updater;

use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use config::Config;
use state::AppState;
use detector::Detector;
use filter::MappingStore;
use proxy::ProxyServer;
use eframe::egui;



#[tokio::main]
async fn main() -> eframe::Result<()> {
    logger::init_logger();

    println!("Starting AI filter...");

    // 1. Load config
    let mut config = Config::load("config.yaml").unwrap_or_else(|_| {
        println!("config.yaml topilmadi, standart sozlamalar ishlatilmoqda.");
        Config {
            proxy: config::ProxyConfig { port: 8081, host: "127.0.0.1".into(), ssl_insecure: false },
            target_sites: vec![
                "chat.openai.com".into(),
                "claude.ai".into(),
                "gemini.google.com".into(),
                "copilot.microsoft.com".into(),
                "grok.x.ai".into(),
                "perplexity.ai".into(),
            ],
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

    // Har doim bo'sh port topib ishlatish (avtomatlashtirish)
    if let Ok(listener) = std::net::TcpListener::bind("127.0.0.1:0") {
        if let Ok(addr) = listener.local_addr() {
            config.proxy.port = addr.port();
        }
    }

    let start_minimized = config.ui.start_minimized;

    // 2. Init state
    let state = Arc::new(AppState::new(config.clone()));
    
    // 3. Init Detector and Mapping Store
    let detector = Arc::new(Detector::new());
    let mapping = MappingStore::new();

    // 4. Background auto-update check
    let update_state = state.clone();
    std::thread::spawn(move || {
        println!("Checking for updates from GitHub...");
        if let Ok(updated) = updater::update_in_background() {
            if updated {
                println!("Update applied! Requires restart.");
                update_state.update_available.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    });

    // 5. Start Proxy Server in background
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



    // 6. Start GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 760.0])
            .with_resizable(false)
            .with_title("AI filter"),
        ..Default::default()
    };
    
    let state_for_cleanup = state.clone();

    let res = eframe::run_native(
        "AI filter",
        options,
        Box::new(move |cc| {
            Ok(Box::new(gui::AIFilterApp::new(
                cc,
                state.clone(),
                start_minimized,
            )))
        }),
    );
    
    // Cleanup proxy immediately on app termination
    state_for_cleanup.sync_system_proxy(false);
    println!("Dastur to'xtatildi, Windows system proxy o'chirildi.");
    res
}
