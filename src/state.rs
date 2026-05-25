use crate::config::Config;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub category: String,
    pub original: String,
    pub masked: String,
}

pub struct AppState {
    pub config: RwLock<Config>,
    pub is_active: AtomicBool,
    
    // Statistika
    pub stats_text_filtered: AtomicUsize,
    pub stats_phone_filtered: AtomicUsize,
    pub stats_email_filtered: AtomicUsize,
    pub stats_image_filtered: AtomicUsize,
    pub stats_total_requests: AtomicUsize,
    
    // Logs
    pub logs: RwLock<Vec<LogEntry>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let state = Self {
            config: RwLock::new(config),
            is_active: AtomicBool::new(true),
            stats_text_filtered: AtomicUsize::new(0),
            stats_phone_filtered: AtomicUsize::new(0),
            stats_email_filtered: AtomicUsize::new(0),
            stats_image_filtered: AtomicUsize::new(0),
            stats_total_requests: AtomicUsize::new(0),
            logs: RwLock::new(Vec::new()),
        };
        state.sync_system_proxy(true);
        state
    }

    pub fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::Relaxed);
        self.sync_system_proxy(active);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    pub fn sync_system_proxy(&self, enable: bool) {
        let conf = self.config.read();
        let port = conf.proxy.port;
        let host = conf.proxy.host.clone();
        
        let sysproxy = sysproxy::Sysproxy {
            enable,
            host,
            port,
            bypass: "localhost;127.*;10.*;172.16.*;192.168.*".into(),
        };
        
        if let Err(e) = sysproxy.set_system_proxy() {
            eprintln!("System proxy sozlashda xatolik: {}", e);
        } else {
            println!("System proxy holati o'zgardi (Yoqilgan: {})", enable);
        }
    }
    
    pub fn add_log(&self, category: &str, original: &str, masked: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let mut logs = self.logs.write();
        logs.push(LogEntry {
            timestamp,
            category: category.to_string(),
            original: original.to_string(),
            masked: masked.to_string(),
        });
        // Max 500 entries
        if logs.len() > 500 {
            logs.remove(0);
        }
    }
}
