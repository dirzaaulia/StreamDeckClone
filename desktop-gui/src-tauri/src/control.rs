use futures_util::{SinkExt, StreamExt};
use tauri::async_runtime::Mutex;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use std::time::Duration;

#[derive(Default)]
pub struct ControlConnection(pub Mutex<Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>);

fn identity_dir() -> Result<std::path::PathBuf, String> {
    Ok(dirs::config_dir().ok_or("Config directory unavailable")?.join("StreamDeckClone"))
}

#[tauri::command]
pub fn host_fingerprint() -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    let directory = identity_dir()?;
    let cert = std::fs::read(directory.join("identity.cert"))
        .map_err(|_| "Host certificate unavailable. Start the host and try again.".to_string())?;
    let key = std::fs::read(directory.join("identity.key"))
        .map_err(|_| "Host identity key unavailable for this user".to_string())?;
    rustls::ServerConfig::builder().with_no_client_auth().with_single_cert(
        vec![CertificateDer::from(cert.clone())], PrivateKeyDer::Pkcs8(key.into()),
    ).map_err(|_| "Host identity is invalid".to_string())?;
    Ok(hex::encode(Sha256::digest(cert)))
}

#[tauri::command]
pub async fn control_disconnect(connection: tauri::State<'_, ControlConnection>) -> Result<(), String> {
    let mut guard = connection.0.lock().await;
    if let Some(mut socket) = guard.take() {
        let _ = socket.close(None).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn control_request(
    message: serde_json::Value,
    connection: tauri::State<'_, ControlConnection>,
) -> Result<serde_json::Value, String> {
    let kind = message.get("type").and_then(|value| value.as_str()).unwrap_or("");
    if !matches!(kind, "get_profile" | "save_profile" | "start_pairing" | "list_devices" | "revoke_device") {
        return Err("Unsupported control request".into());
    }
    let serialized = serde_json::to_string(&message).map_err(|error| error.to_string())?;
    if serialized.len() > 8192 {
        return Err("Control request too large".into());
    }
    let mut guard = tokio::time::timeout(Duration::from_secs(5), connection.0.lock()).await
        .map_err(|_| "Host control connection is busy")?;
    if guard.is_none() {
        host_fingerprint()?;
        let secret = std::fs::read_to_string(identity_dir()?.join("control-secret"))
            .map_err(|_| "Host control credentials unavailable for this user".to_string())?;
        if secret.len() != 64 || !secret.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("Host control credentials are invalid".into());
        }
        let mut request = "ws://127.0.0.1:4456".into_client_request().map_err(|error| error.to_string())?;
        request.headers_mut().insert("origin", "tauri://localhost".parse().unwrap());
        request.headers_mut().insert("authorization", format!("Bearer {secret}").parse().map_err(|_| "Invalid credentials")?);
        let connection_result = tokio::time::timeout(std::time::Duration::from_secs(5),
            tokio_tungstenite::connect_async(request)).await
            .map_err(|_| "Host control connection timed out")?
            .map_err(|_| "Host rejected desktop controls. Check that the host runs as the same user.".to_string())?;
        *guard = Some(connection_result.0);
    }
    let socket = guard.as_mut().unwrap();
    let result = async {
        socket.send(Message::Text(serialized.into())).await.map_err(|error| error.to_string())?;
        loop {
            match socket.next().await {
                Some(Ok(Message::Text(text))) => return serde_json::from_str(&text).map_err(|error| error.to_string()),
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => continue,
                Some(Ok(_)) | None => return Err("Host control connection closed".into()),
                Some(Err(error)) => return Err(error.to_string()),
            }
        }
    };
    match tokio::time::timeout(std::time::Duration::from_secs(5), result).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => { *guard = None; Err(error) },
        Err(_) => { *guard = None; Err("Host control request timed out".into()) },
    }
}
