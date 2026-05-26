use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::state::AppState;
use crate::detector::{Detector, DetectionType};
use crate::filter::{MappingStore, MultiFilter};
use std::sync::atomic::Ordering;
use crate::mitm::MitmManager;

pub struct ProxyServer {
    port: u16,
    state: Arc<AppState>,
    detector: Arc<Detector>,
    mapping: Arc<MappingStore>,
    target_sites: Vec<String>,
    mitm: Arc<MitmManager>,
    client_config: Arc<rustls::ClientConfig>,
}

impl ProxyServer {
    pub fn new(
        port: u16,
        state: Arc<AppState>,
        detector: Arc<Detector>,
        mapping: Arc<MappingStore>,
        target_sites: Vec<String>,
    ) -> Self {
        // TLS Client Config for upstream
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let client_config = Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        );

        Self {
            port,
            state,
            detector,
            mapping,
            target_sites,
            mitm: Arc::new(MitmManager::new()),
            client_config,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("🔒 AI filter is listening on {} (WITH HTTPS MITM)", addr);

        loop {
            let (client_stream, _) = listener.accept().await?;
            let state = self.state.clone();
            let detector = self.detector.clone();
            let mapping = self.mapping.clone();
            let target_sites = self.target_sites.clone();
            let mitm = self.mitm.clone();
            let client_config = self.client_config.clone();

            tokio::spawn(async move {
                let _ = handle_connection(
                    client_stream, state, detector, mapping,
                    target_sites, mitm, client_config,
                ).await;
            });
        }
    }
}

// =====================================================
// HTTP BODY PARSING — faqat body ni filterlaydi, header tegmaydi
// =====================================================

/// HTTP payloaddan faqat body ni ajratib filterlash
fn filter_http_body(
    payload: &[u8],
    state: &Arc<AppState>,
    detector: &Arc<Detector>,
    mapping: &Arc<MappingStore>,
    domain: &str,
) -> Vec<u8> {
    if !state.is_active() {
        return payload.to_vec();
    }

    // Bu domen uchun platforma yoqilganmi?
    let platform = match state.platform_for_host(domain) {
        Some(p) => p,
        None => return payload.to_vec(),
    };

    let text = String::from_utf8_lossy(payload).to_string();

    // HTTP so'rovda header va body ni ajratish
    let (headers_part, body_part) = if let Some(idx) = text.find("\r\n\r\n") {
        (&text[..idx + 4], &text[idx + 4..])
    } else {
        return payload.to_vec(); // body topilmadi
    };

    // Faqat content-type: application/json bo'lsa filterlash
    let ct_lower = headers_part.to_lowercase();
    let is_json = ct_lower.contains("application/json");
    let is_text = ct_lower.contains("text/");
    let is_form = ct_lower.contains("application/x-www-form-urlencoded");

    if !is_json && !is_text && !is_form {
        return payload.to_vec(); // binary, rasm, kabi — tegmaydi
    }

    if body_part.trim().is_empty() {
        return payload.to_vec();
    }

    // BODY ni filterlash
    let filter = MultiFilter::new(mapping.clone(), "label".to_string());
    let detections = detector.detect_text(body_part, &[]);

    if detections.is_empty() {
        return payload.to_vec();
    }

    let masked_body = filter.mask_text(body_part, &detections);

    // Statistikani yangilash
    for det in &detections {
        match det.dtype {
            DetectionType::Phone => {
                state.stats_phone_filtered.fetch_add(1, Ordering::Relaxed);
                state.add_log("Telefon", &det.original_value, det.dtype.as_label());
            },
            DetectionType::Email => {
                state.stats_email_filtered.fetch_add(1, Ordering::Relaxed);
                state.add_log("Email", &det.original_value, det.dtype.as_label());
            },
            _ => {
                state.stats_text_filtered.fetch_add(1, Ordering::Relaxed);
                state.add_log("Matn/PII", &det.original_value, det.dtype.as_label());
            }
        }
    }

    println!(
        "✅ [{}] {} ta shaxsiy ma'lumot {} trafikdan filterlandi!",
        platform,
        detections.len(),
        domain
    );

    // Content-Length ni yangilash
    let mut new_headers = String::new();
    for line in headers_part.lines() {
        if line.to_lowercase().starts_with("content-length:") {
            new_headers.push_str(&format!("Content-Length: {}\r\n", masked_body.len()));
        } else if !line.is_empty() {
            new_headers.push_str(line);
            new_headers.push_str("\r\n");
        }
    }
    new_headers.push_str("\r\n");

    let mut result = new_headers.into_bytes();
    result.extend_from_slice(masked_body.as_bytes());
    result
}

/// Javob (response) dagi tokenlarni asl qiymatga tiklash
fn restore_http_body(
    payload: &[u8],
    mapping: &Arc<MappingStore>,
) -> Vec<u8> {
    let text = String::from_utf8_lossy(payload).to_string();

    let (headers_part, body_part) = if let Some(idx) = text.find("\r\n\r\n") {
        (&text[..idx + 4], &text[idx + 4..])
    } else {
        return payload.to_vec();
    };

    if body_part.trim().is_empty() {
        return payload.to_vec();
    }

    let filter = MultiFilter::new(mapping.clone(), "label".to_string());
    let restored_body = filter.restore_text(body_part);

    if restored_body == body_part {
        return payload.to_vec(); // hech narsa o'zgarmadi
    }

    // Content-Length ni yangilash
    let mut new_headers = String::new();
    for line in headers_part.lines() {
        if line.to_lowercase().starts_with("content-length:") {
            new_headers.push_str(&format!("Content-Length: {}\r\n", restored_body.len()));
        } else if !line.is_empty() {
            new_headers.push_str(line);
            new_headers.push_str("\r\n");
        }
    }
    new_headers.push_str("\r\n");

    let mut result = new_headers.into_bytes();
    result.extend_from_slice(restored_body.as_bytes());
    result
}


// =====================================================
// CONNECTION HANDLER
// =====================================================

async fn handle_connection(
    mut client_stream: TcpStream,
    state: Arc<AppState>,
    detector: Arc<Detector>,
    mapping: Arc<MappingStore>,
    target_sites: Vec<String>,
    mitm: Arc<MitmManager>,
    client_config: Arc<rustls::ClientConfig>,
) -> anyhow::Result<()> {
    let mut buffer = vec![0u8; 16384];
    
    let n = client_stream.read(&mut buffer).await?;
    if n == 0 { return Ok(()); }

    state.stats_total_requests.fetch_add(1, Ordering::Relaxed);
    
    let request_str = String::from_utf8_lossy(&buffer[..n]).to_string();
    let first_line = request_str.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    // ==== HTTPS CONNECT Mode (MITM) ====
    if parts.len() >= 2 && parts[0] == "CONNECT" {
        let target = parts[1];
        let domain = target.split(':').next().unwrap_or(target).to_string();
        
        let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
        client_stream.write_all(response.as_bytes()).await?;
        
        // Bu domen uchun filtr yoqilganmi tekshirish
        let should_mitm = state.platform_for_host(&domain).is_some();

        if should_mitm {
            // 1. MITM TLS server yaratish
            if let Ok(server_config) = mitm.get_server_config(&domain).await {
                let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
                
                if let Ok(mut secure_client) = acceptor.accept(client_stream).await {
                    // 2. Upstream serverga ulanish
                    if let Ok(upstream) = TcpStream::connect(target).await {
                        let server_name = match rustls::pki_types::ServerName::try_from(domain.clone()) {
                            Ok(sn) => sn.to_owned(),
                            Err(_) => return Ok(()),
                        };
                        
                        let connector = tokio_rustls::TlsConnector::from(client_config);
                        if let Ok(mut secure_upstream) = connector.connect(server_name, upstream).await {
                            
                            // REQUEST ni o'qish va filterlash
                            let mut secure_buf = vec![0u8; 65536];
                            let secure_c_read = tokio::time::timeout(
                                tokio::time::Duration::from_millis(500),
                                secure_client.read(&mut secure_buf)
                            ).await;
                            
                            if let Ok(Ok(sn)) = secure_c_read {
                                if sn > 0 {
                                    // FILTERLASH: so'rov body dagi PII ni maskalash
                                    let filtered = filter_http_body(
                                        &secure_buf[..sn], &state, &detector, &mapping, &domain
                                    );
                                    secure_upstream.write_all(&filtered).await?;
                                }
                            }

                            // RESPONSE ni o'qish va tiklash
                            let mut resp_buf = vec![0u8; 65536];
                            let resp_read = tokio::time::timeout(
                                tokio::time::Duration::from_secs(30),
                                secure_upstream.read(&mut resp_buf)
                            ).await;

                            if let Ok(Ok(rn)) = resp_read {
                                if rn > 0 {
                                    // TIKLASH: javobdagi tokenlarni asl qiymatga qaytarish
                                    let restored = restore_http_body(&resp_buf[..rn], &mapping);
                                    secure_client.write_all(&restored).await?;
                                }
                            }

                            // Keyingi so'rovlarni ham filterlash (bidirectional)
                            let state_c2s = state.clone();
                            let det_c2s = detector.clone();
                            let map_c2s = mapping.clone();
                            let domain_c2s = domain.clone();
                            let map_s2c = mapping.clone();

                            let (mut c_read, mut c_write) = tokio::io::split(secure_client);
                            let (mut s_read, mut s_write) = tokio::io::split(secure_upstream);
                            
                            // Client → Server (filterlash)
                            let c2s = tokio::spawn(async move {
                                let mut buf = vec![0u8; 65536];
                                loop {
                                    match c_read.read(&mut buf).await {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            let filtered = filter_http_body(
                                                &buf[..n], &state_c2s, &det_c2s, &map_c2s, &domain_c2s
                                            );
                                            if s_write.write_all(&filtered).await.is_err() { break; }
                                        }
                                    }
                                }
                            });
                            
                            // Server → Client (tiklash)
                            let s2c = tokio::spawn(async move {
                                let mut buf = vec![0u8; 65536];
                                loop {
                                    match s_read.read(&mut buf).await {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            let restored = restore_http_body(&buf[..n], &map_s2c);
                                            if c_write.write_all(&restored).await.is_err() { break; }
                                        }
                                    }
                                }
                            });
                            
                            let _ = tokio::try_join!(c2s, s2c);
                        }
                    }
                }
            }
        } else {
            // Filtr o'chirilgan saytlar — oddiy tunnel
            if let Ok(mut upstream) = TcpStream::connect(target).await {
                let (mut c_read, mut c_write) = tokio::io::split(client_stream);
                let (mut s_read, mut s_write) = tokio::io::split(upstream);
                
                let c2s = tokio::spawn(async move { let _ = tokio::io::copy(&mut c_read, &mut s_write).await; });
                let s2c = tokio::spawn(async move { let _ = tokio::io::copy(&mut s_read, &mut c_write).await; });
                
                let _ = tokio::try_join!(c2s, s2c);
            }
        }
        return Ok(());
    }
    
    // ==== PAC Script ====
    if parts.len() >= 3 {
        let url = parts[1];
        
        if url == "/pac" || url.ends_with("/pac") {
            let mut pac_conditions = String::new();
            // PLATFORM_HOSTS dan barcha yoqilgan platformalar hostlarini yig'ish
            for (name, hosts) in crate::state::PLATFORM_HOSTS {
                if state.is_platform_enabled(name) {
                    for host in *hosts {
                        pac_conditions.push_str(&format!("shExpMatch(host, '*{}*') || ", host));
                    }
                }
            }
            if pac_conditions.ends_with(" || ") {
                pac_conditions.truncate(pac_conditions.len() - 4);
            }
            if pac_conditions.is_empty() {
                pac_conditions = "false".to_string();
            }

            let port = state.config.read().proxy.port;
            let pac_content = format!(
                "function FindProxyForURL(url, host) {{\n    if ({}) {{\n        return 'PROXY 127.0.0.1:{}';\n    }}\n    return 'DIRECT';\n}}",
                pac_conditions, port
            );
            
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nConnection: close\r\n\r\n{}",
                pac_content
            );
            client_stream.write_all(response.as_bytes()).await?;
            return Ok(());
        }

        // Oddiy HTTP so'rov
        let host = extract_host(&request_str, url);
        
        if let Some(host) = host {
            let processed = filter_http_body(&buffer[..n], &state, &detector, &mapping, &host);
            
            let port = if url.starts_with("http://") { 80 } else { 443 };
            let connect_addr = format!("{}:{}", host, port);
            
            match TcpStream::connect(&connect_addr).await {
                Ok(mut upstream) => {
                    upstream.write_all(&processed).await?;
                    let mut response_buf = vec![0u8; 65536];
                    let response_n = upstream.read(&mut response_buf).await.unwrap_or(0);
                    if response_n > 0 {
                        let restored = restore_http_body(&response_buf[..response_n], &mapping);
                        client_stream.write_all(&restored).await?;
                    }
                },
                Err(_) => {
                    let error_response = format!(
                        "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/html\r\n\r\n<h1>AI filter</h1><p>Serverga ulanib bo'lmadi: {}</p>",
                        connect_addr
                    );
                    client_stream.write_all(error_response.as_bytes()).await?;
                }
            }
        } else {
            let info_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
                <html><body><h1>🛡️ AI filter v1.2.3</h1><p>Proksi faol ishlamoqda!</p></body></html>"
            );
            client_stream.write_all(info_response.as_bytes()).await?;
        }
    }

    Ok(())
}

fn extract_host(request: &str, url: &str) -> Option<String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        if let Some(after_scheme) = url.split("://").nth(1) {
            let host = after_scheme.split('/').next().unwrap_or("");
            let host = host.split(':').next().unwrap_or(host);
            if !host.is_empty() { return Some(host.to_string()); }
        }
    }
    for line in request.lines() {
        if line.to_lowercase().starts_with("host:") {
            let host = line[5..].trim();
            let host = host.split(':').next().unwrap_or(host);
            return Some(host.to_string());
        }
    }
    None
}
