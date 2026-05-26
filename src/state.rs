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
    
    // Updates
    pub update_available: AtomicBool,
}

use winreg::enums::*;
use winreg::RegKey;
use std::ptr;

#[link(name = "wininet")]
extern "system" {
    fn InternetSetOptionW(
        hInternet: *mut std::ffi::c_void,
        dwOption: u32,
        lpBuffer: *mut std::ffi::c_void,
        dwBufferLength: u32,
    ) -> i32;
}

const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
const INTERNET_OPTION_REFRESH: u32 = 37;

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
            update_available: AtomicBool::new(false),
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
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", KEY_SET_VALUE) {
            if enable {
                let pac_url = format!("http://127.0.0.1:{}/pac", port);
                let _ = key.set_value("AutoConfigURL", &pac_url);
                let _ = key.set_value("ProxyEnable", &0u32);
            } else {
                let _ = key.delete_value("AutoConfigURL");
                let _ = key.set_value("ProxyEnable", &0u32);
            }
            
            unsafe {
                InternetSetOptionW(ptr::null_mut(), INTERNET_OPTION_SETTINGS_CHANGED, ptr::null_mut(), 0);
                InternetSetOptionW(ptr::null_mut(), INTERNET_OPTION_REFRESH, ptr::null_mut(), 0);
            }
        }
        
        println!("System proxy (PAC) holati o'zgardi (Yoqilgan: {})", enable);
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
