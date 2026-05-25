use rcgen::{CertificateParams, KeyPair, Certificate};
use std::fs;
use std::path::Path;
use rustls::pki_types::CertificateDer;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Stores the CA's key pair + self-signed cert so we can sign leaf certs on the fly.
pub struct MitmManager {
    ca_key: KeyPair,
    ca_cert: Certificate,
    cert_cache: RwLock<HashMap<String, Arc<rustls::ServerConfig>>>,
}

impl MitmManager {
    pub fn new() -> Self {
        let ca_dir = Path::new("ca");
        if !ca_dir.exists() {
            fs::create_dir_all(ca_dir).unwrap();
        }

        let key_path = ca_dir.join("root.key");
        let cert_path = ca_dir.join("root.crt");

        // Always generate fresh CA params
        let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.distinguished_name.push(rcgen::DnType::CommonName, "PrivacyProxy Root CA");
        ca_params.distinguished_name.push(rcgen::DnType::OrganizationName, "PrivacyProxy");

        let (ca_key, ca_cert) = if key_path.exists() && cert_path.exists() {
            // Load existing key
            let key_pem = fs::read_to_string(&key_path).unwrap();
            let key = KeyPair::from_pem(&key_pem).unwrap();
            
            // Re-sign with same key to get a Certificate object
            let cert = ca_params.self_signed(&key).unwrap();
            println!("🔑 Mavjud Root CA yuklandi.");
            (key, cert)
        } else {
            // Generate new key + CA
            println!("🔐 Yangi Root CA sertifikati yaratilmoqda...");
            let key = KeyPair::generate().unwrap();
            let cert = ca_params.self_signed(&key).unwrap();
            
            // Save key and cert in PEM
            fs::write(&key_path, key.serialize_pem()).unwrap();
            fs::write(&cert_path, cert.pem()).unwrap();
            
            println!("✅ Root CA yaratildi: {:?}", cert_path);
            println!("⚠  Brauzer ishonishi uchun bu faylni \"Ishonchli sertifikatlar\" ga qo'shing!");
            (key, cert)
        };

        Self {
            ca_key,
            ca_cert,
            cert_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Generate (or retrieve cached) a rustls ServerConfig for intercepting `domain`
    pub async fn get_server_config(&self, domain: &str) -> anyhow::Result<Arc<rustls::ServerConfig>> {
        // Check cache
        {
            let cache = self.cert_cache.read().await;
            if let Some(cfg) = cache.get(domain) {
                return Ok(cfg.clone());
            }
        }

        // Generate a leaf certificate signed by our CA
        let leaf_key = KeyPair::generate()?;
        
        let mut leaf_params = CertificateParams::new(vec![domain.to_string()])?;
        leaf_params.distinguished_name.push(rcgen::DnType::CommonName, domain);

        // Sign the leaf with our CA key + cert
        let leaf_cert = leaf_params.signed_by(&leaf_key, &self.ca_cert, &self.ca_key)?;
        
        let leaf_cert_der = leaf_cert.der().clone();
        let ca_cert_der = self.ca_cert.der().clone();
        let leaf_key_der = leaf_key.serialize_der();

        // Build rustls ServerConfig with cert chain [leaf, ca]
        let cert_chain = vec![leaf_cert_der, ca_cert_der];
        let private_key = rustls::pki_types::PrivateKeyDer::try_from(leaf_key_der)
            .map_err(|e| anyhow::anyhow!("Key conversion error: {:?}", e))?;
        
        let mut config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];

        let arc_cfg = Arc::new(config);
        
        // Cache it
        let mut cache = self.cert_cache.write().await;
        cache.insert(domain.to_string(), arc_cfg.clone());
        
        println!("🔑 MITM sertifikat yaratildi: {}", domain);
        Ok(arc_cfg)
    }
}
