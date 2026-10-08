use rcgen::generate_simple_self_signed;
use ring::rand::{SecureRandom, SystemRandom};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct Identity {
    pub config: rustls::ServerConfig,
    pub fingerprint: String,
    pub control_secret: String,
}

pub fn config_dir() -> PathBuf {
    crate::config::AppConfig::config_path()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::fs::OpenOptions;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Windows CREATE_NEW avoids replacing another user's existing secret.
        options.share_mode(0);
    }
    use std::io::Write;
    options.open(path)?.write_all(bytes)
}

pub fn load_identity() -> Result<Identity, Box<dyn std::error::Error>> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let cert_path = dir.join("identity.cert");
    let key_path = dir.join("identity.key");
    if cert_path.exists() != key_path.exists() {
        return Err("Incomplete TLS identity; restore both identity files".into());
    }
    if !cert_path.exists() {
        let cert = generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])?;
        write_private(&key_path, &cert.key_pair.serialize_der())?;
        if let Err(error) = write_private(&cert_path, cert.cert.der().as_ref()) {
            let _ = fs::remove_file(&key_path);
            return Err(error.into());
        }
    }
    let cert_der = fs::read(&cert_path)?;
    let key_der = fs::read(&key_path)?;
    let fingerprint = hex::encode(Sha256::digest(&cert_der));
    let cert = CertificateDer::from(cert_der);
    let key = PrivateKeyDer::Pkcs8(key_der.into());
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    let secret_path = dir.join("control-secret");
    if !secret_path.exists() {
        let mut bytes = [0u8; 32];
        SystemRandom::new()
            .fill(&mut bytes)
            .map_err(|_| "Randomness unavailable")?;
        write_private(&secret_path, hex::encode(bytes).as_bytes())?;
    }
    let control_secret = fs::read_to_string(&secret_path)?;
    if control_secret.len() != 64 || hex::decode(&control_secret)?.len() != 32 {
        return Err("Invalid control secret".into());
    }
    Ok(Identity {
        config,
        fingerprint,
        control_secret,
    })
}
