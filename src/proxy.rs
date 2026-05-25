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
        println!("🔒 PrivacyProxy is listening on {} (WITH HTTPS MITM)", addr);

        loop {
            let (client_stream, client_addr) = listener.accept().await?;
            let state = self.state.clone();
            let detector = self.detector.clone();
            let mapping = self.mapping.clone();
            let target_sites = self.target_sites.clone();
            let mitm = self.mitm.clone();
            let client_config = self.client_config.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(client_stream, state, detector, mapping, target_sites, mitm, client_config).await {
                    // Ignore noisy errors like broken pipes or resets
                }
            });
        }
    }
}

// Function that parses an HTTP payload, filters PII, and patches Content-Length
fn process_http_payload(
    payload: &[u8],
    state: &Arc<AppState>,
    detector: &Arc<Detector>,
    mapping: &Arc<MappingStore>,
) -> Vec<u8> {
    if !state.is_active() {
        return payload.to_vec();
    }
    
    // We only process if it looks like UTF-8 text (or we fallback lossy)
    let text = String::from_utf8_lossy(payload).to_string();
    
    let filter = MultiFilter::new(mapping.clone(), "label".to_string());
    let detections = detector.detect_text(&text, &[]);
    
    if detections.is_empty() {
        return payload.to_vec();
    }
    
    let mut masked_text = filter.mask_text(&text, &detections);
    
    for det in &detections {
        match det.dtype {
            DetectionType::Phone => { state.stats_phone_filtered.fetch_add(1, Ordering::Relaxed); state.add_log("Telefon", &det.original_value, det.dtype.as_label()); },
            DetectionType::Email => { state.stats_email_filtered.fetch_add(1, Ordering::Relaxed); state.add_log("Email", &det.original_value, det.dtype.as_label()); },
            _ => { state.stats_text_filtered.fetch_add(1, Ordering::Relaxed); state.add_log("Matn/PII", &det.original_value, det.dtype.as_label()); }
        }
    }
    println!("✅ {} ta shaxsiy ma'lumot HTTP trafikdan ushlab qolindi!", detections.len());
    
    // Patch Content-Length if it exists
    if let Some(body_idx) = masked_text.find("\r\n\r\n") {
        let (headers_part, body_part) = masked_text.split_at(body_idx + 4);
        let new_len = body_part.len();
        
        // Find and replace Content-Length in headers
        let mut new_headers = String::new();
        for line in headers_part.lines() {
            if line.to_lowercase().starts_with("content-length:") {
                new_headers.push_str(&format!("Content-Length: {}\r\n", new_len));
            } else if !line.is_empty() {
                new_headers.push_str(line);
                new_headers.push_str("\r\n");
            }
        }
        new_headers.push_str("\r\n");
        new_headers.push_str(body_part);
        return new_headers.into_bytes();
    }
    
    masked_text.into_bytes()
}

async fn handle_connection(
    mut client_stream: TcpStream,
    state: Arc<AppState>,
    detector: Arc<Detector>,
    mapping: Arc<MappingStore>,
    _target_sites: Vec<String>,
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
    
    // ==== Handle HTTPS CONNECT Mode (MITM) ====
    if parts.len() >= 2 && parts[0] == "CONNECT" {
        let target = parts[1];
        let domain = target.split(':').next().unwrap_or(target).to_string();
        
        let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
        client_stream.write_all(response.as_bytes()).await?;
        
        // 1. Establish TLS server with local client
        if let Ok(server_config) = mitm.get_server_config(&domain).await {
            let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
            
            if let Ok(mut secure_client) = acceptor.accept(client_stream).await {
                // 2. Connect to actual upstream
                if let Ok(upstream) = TcpStream::connect(target).await {
                    let server_name = match rustls::pki_types::ServerName::try_from(domain.clone()) {
                        Ok(sn) => sn.to_owned(),
                        Err(_) => return Ok(()),
                    };
                    
                    let connector = tokio_rustls::TlsConnector::from(client_config);
                    if let Ok(mut secure_upstream) = connector.connect(server_name, upstream).await {
                        
                        // We must read the first payload from secure_client manually to intercept it!
                        let mut secure_buf = vec![0u8; 32768];
                        let mut secure_c_read = tokio::time::timeout(tokio::time::Duration::from_millis(500), secure_client.read(&mut secure_buf)).await;
                        
                        if let Ok(Ok(sn)) = secure_c_read {
                            if sn > 0 {
                                // MASK AND FILTER THE HTTPS PAYLOAD!
                                let processed_request = process_http_payload(&secure_buf[..sn], &state, &detector, &mapping);
                                // Send patched request to upstream
                                secure_upstream.write_all(&processed_request).await?;
                            }
                        }

                        // Now tunnel the rest blindly (or we could loop process_http_payload for full session, but simple tunneling is safer for chunking for now)
                        let (mut c_read, mut c_write) = tokio::io::split(secure_client);
                        let (mut s_read, mut s_write) = tokio::io::split(secure_upstream);
                        
                        let c2s = tokio::spawn(async move { let _ = tokio::io::copy(&mut c_read, &mut s_write).await; });
                        let s2c = tokio::spawn(async move { let _ = tokio::io::copy(&mut s_read, &mut c_write).await; });
                        
                        let _ = tokio::try_join!(c2s, s2c);
                    }
                }
            }
        }
        return Ok(());
    }
    
    // ==== Handle DIRECT HTTP Mode ====
    if parts.len() >= 3 {
        let url = parts[1];
        let host = extract_host(&request_str, url);
        
        if let Some(host) = host {
            let processed_buffer = process_http_payload(&buffer[..n], &state, &detector, &mapping);
            
            let port = if url.starts_with("http://") { 80 } else { 443 };
            let connect_addr = format!("{}:{}", host, port);
            
            match TcpStream::connect(&connect_addr).await {
                Ok(mut upstream) => {
                    upstream.write_all(&processed_buffer).await?;
                    let mut response_buf = vec![0u8; 65536];
                    let response_n = upstream.read(&mut response_buf).await.unwrap_or(0);
                    if response_n > 0 {
                        client_stream.write_all(&response_buf[..response_n]).await?;
                    }
                },
                Err(_) => {
                    let error_response = format!(
                        "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/html\r\n\r\n<h1>PrivacyProxy</h1><p>Serverga ulanib bo'lmadi: {}</p>",
                        connect_addr
                    );
                    client_stream.write_all(error_response.as_bytes()).await?;
                }
            }
        } else {
            let info_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
                <html><body><h1>🛡️ PrivacyProxy v1.0.0</h1><p>Proksi faol ishlamoqda!</p></body></html>"
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
