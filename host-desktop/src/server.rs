use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use prost::Message;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use subtle::ConstantTimeEq;
use tracing::{error, info, warn};

use crate::audio::AudioController;
use crate::config::{AppConfig, KeyConfig};
use crate::input::InputExecutor;
use crate::layout::{create_layout_message, current_time_millis};
use crate::pairing::{PairingStore, pairing_path};
use crate::profiles::ProfileWatcher;
use crate::protocol::streamdeck::{
    DeckMessage, HandshakeRequest, HandshakeResponse, Heartbeat, KeyEventType, deck_message,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ControlMessage {
    #[serde(rename = "get_profile")]
    GetProfile { profile: String },
    #[serde(rename = "save_profile")]
    SaveProfile {
        profile: String,
        keys: Vec<KeyConfig>,
    },
    #[serde(rename = "start_pairing")]
    StartPairing,
    #[serde(rename = "list_devices")]
    ListDevices,
    #[serde(rename = "revoke_device")]
    RevokeDevice { device_id: String },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ControlResponse {
    #[serde(rename = "profile")]
    Profile {
        profile: String,
        keys: Vec<KeyConfig>,
    },
    #[serde(rename = "saved")]
    Saved { profile: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "pairing_code")]
    PairingCode { code: String },
    #[serde(rename = "devices")]
    Devices { device_ids: Vec<String> },
}

pub struct DeckServer {
    input_executor: Arc<InputExecutor>,
    #[allow(dead_code)]
    audio: Arc<Mutex<AudioController>>,
    config: Arc<Mutex<AppConfig>>,
    pairing: Arc<Mutex<PairingStore>>,
    active_profile: Arc<Mutex<String>>,
    broadcast_tx: tokio::sync::broadcast::Sender<DeckMessage>,
}

impl DeckServer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let audio = AudioController::new().unwrap_or_else(|_| AudioController::stub());
        let input_executor = Arc::new(InputExecutor::new(audio.clone()));
        let config = AppConfig::load();
        let pairing = PairingStore::load(&pairing_path())?;
        let (broadcast_tx, _) = tokio::sync::broadcast::channel(16);

        Ok(Self {
            input_executor,
            audio,
            config: Arc::new(Mutex::new(config)),
            pairing: Arc::new(Mutex::new(pairing)),
            active_profile: Arc::new(Mutex::new("Default".to_string())),
            broadcast_tx,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        ProfileWatcher::spawn(tx);

        let config_clone = self.config.clone();
        let active_profile_clone = self.active_profile.clone();
        let broadcast_tx_clone = self.broadcast_tx.clone();

        tokio::spawn(async move {
            while let Some(mut profile) = rx.recv().await {
                // Ensure profile exists in config, otherwise fallback to Default
                {
                    let cfg = config_clone.lock().unwrap();
                    if !cfg.profiles.contains_key(&profile) {
                        profile = "Default".to_string();
                    }
                }

                let mut current = active_profile_clone.lock().unwrap();
                if *current != profile {
                    *current = profile.clone();
                    info!("Active profile switched to: {}", profile);

                    // Broadcast new layout
                    let cfg = config_clone.lock().unwrap();
                    if let Some(prof) = cfg.profiles.get(&profile) {
                        let layout_msg = create_layout_message(&profile, prof);
                        let _ = broadcast_tx_clone.send(layout_msg);
                    }
                }
            }
        });

        let identity = crate::tls::load_identity()?;
        info!("Host certificate SHA-256 fingerprint: {}", identity.fingerprint);
        let phone = TcpListener::bind("0.0.0.0:4455").await?;
        let control = TcpListener::bind("127.0.0.1:4456").await?;
        let listeners = vec![(phone, false), (control, true)];
        let mut tasks = Vec::new();
        let tls = Arc::new(tokio_rustls::TlsAcceptor::from(Arc::new(identity.config)));
        for (listener, control_channel) in listeners {
            let tls = tls.clone();
            let connection_limit = Arc::new(tokio::sync::Semaphore::new(if control_channel { 4 } else { 64 }));
            let mut peer_limits = std::collections::HashMap::new();
            let control_secret = identity.control_secret.clone();
            let executor = Arc::clone(&self.input_executor);
            let cfg = Arc::clone(&self.config);
            let pairing = Arc::clone(&self.pairing);
            let active_profile = Arc::clone(&self.active_profile);
            let btx = self.broadcast_tx.clone();

            tasks.push(tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, peer_addr)) => {
                            peer_limits.retain(|_, limit: &mut Arc<tokio::sync::Semaphore>| {
                                limit.available_permits() < 4
                            });
                            let peer_limit = peer_limits.entry(peer_addr.ip())
                                .or_insert_with(|| Arc::new(tokio::sync::Semaphore::new(4)))
                                .clone();
                            let (Ok(global_permit), Ok(peer_permit)) = (
                                connection_limit.clone().try_acquire_owned(),
                                peer_limit.try_acquire_owned(),
                            ) else {
                                continue;
                            };
                            let exec = Arc::clone(&executor);
                            let c = Arc::clone(&cfg);
                            let p = Arc::clone(&pairing);
                            let ap = Arc::clone(&active_profile);
                            let tx = btx.clone();
                            let rx = btx.subscribe();

                            let acceptor = tls.clone();
                            let secret = control_secret.clone();
                            tokio::spawn(async move {
                                let _permits = (global_permit, peer_permit);
                                let result = if control_channel {
                                    handle_connection(stream, peer_addr, exec, c, p, ap, tx, rx, pairing_path(), Some(secret)).await
                                } else {
                                    match tokio::time::timeout(std::time::Duration::from_secs(5), acceptor.accept(stream)).await {
                                        Ok(Ok(secured)) => handle_connection(secured, peer_addr, exec, c, p, ap, tx, rx, pairing_path(), None).await,
                                        _ => return,
                                    }
                                };
                                if let Err(e) = result { warn!("Connection error with {}: {}", peer_addr, e); }
                            });
                        }
                        Err(e) => error!("Error accepting connection: {}", e),
                    }
                }
            }));
        }

        futures_util::future::join_all(tasks).await;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_connection(
    stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    peer_addr: SocketAddr,
    input_executor: Arc<InputExecutor>,
    config: Arc<Mutex<AppConfig>>,
    pairing: Arc<Mutex<PairingStore>>,
    active_profile: Arc<Mutex<String>>,
    broadcast_tx: tokio::sync::broadcast::Sender<DeckMessage>,
    mut broadcast_rx: tokio::sync::broadcast::Receiver<DeckMessage>,
    pairing_file: std::path::PathBuf,
    control_secret: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let trusted = control_secret.is_some();
    #[allow(clippy::result_large_err)]
    let callback = move |req: &Request, response: Response| {
        if let Some(secret) = &control_secret {
            let expected = format!("Bearer {secret}");
            let provided = req.headers().get("authorization")
                .and_then(|value| value.to_str().ok()).unwrap_or("");
            let origin = req.headers().get("origin").and_then(|value| value.to_str().ok()).unwrap_or("");
            let allowed = matches!(origin, "http://localhost:1420" | "http://127.0.0.1:1420" | "tauri://localhost" | "http://tauri.localhost");
            if !allowed || provided.as_bytes().ct_eq(expected.as_bytes()).unwrap_u8() != 1 {
                return Err(tokio_tungstenite::tungstenite::http::Response::builder().status(403)
                    .body(Some("Forbidden".to_string())).unwrap());
            }
        } else if req.headers().get("sec-websocket-protocol").is_some() {
            return Err(tokio_tungstenite::tungstenite::http::Response::builder().status(403)
                .body(Some("Forbidden".to_string())).unwrap());
        }
        Ok(response)
    };
    let settings = WebSocketConfig::default().max_message_size(Some(65536)).max_frame_size(Some(65536));
    let ws_stream = tokio::time::timeout(std::time::Duration::from_secs(5),
        tokio_tungstenite::accept_hdr_async_with_config(stream, callback, Some(settings))).await??;
    info!("Accepted {} WebSocket connection from {}", if trusted { "control" } else { "phone" }, peer_addr);

    let (mut write, mut read) = ws_stream.split();
    let mut deck_session: Option<(String, String)> = None;
    let mut authorization_check = tokio::time::interval(std::time::Duration::from_secs(1));
    let handshake_deadline = tokio::time::sleep(std::time::Duration::from_secs(10));
    tokio::pin!(handshake_deadline);

    loop {
        tokio::select! {
            _ = &mut handshake_deadline, if !trusted && deck_session.is_none() => {
                warn!("Phone handshake timed out");
                break;
            }
            _ = authorization_check.tick() => {
                if let Some((device_id, token)) = &deck_session
                    && !pairing.lock().unwrap().is_authorized(device_id, token) {
                        warn!("Paired device was removed; closing connection");
                        let _ = write.send(WsMessage::Close(None)).await;
                        break;
                    }
            }
            msg_result = read.next() => {
                match msg_result {
                    Some(Ok(msg)) => {
                        match msg {
                            WsMessage::Binary(bin_data) => {
                                if trusted {
                                    warn!("Rejected binary message from trusted GUI client");
                                    continue;
                                }
                                match DeckMessage::decode(bin_data.as_ref()) {
                                    Ok(deck_msg) => {
                                        if let Some(deck_message::Payload::HandshakeReq(req)) = deck_msg.payload.as_ref() {
                                            if deck_session.is_some() {
                                                let _ = write.send(WsMessage::Close(None)).await;
                                                break;
                                            }
                                            deck_session = handle_handshake(req, &mut write, &config, &pairing, &active_profile, &pairing_file).await?;
                                            if deck_session.is_none() {
                                                let _ = write.send(WsMessage::Close(None)).await;
                                                break;
                                            }
                                        } else if let Some((device_id, token)) = &deck_session {
                                            if pairing.lock().unwrap().is_authorized(device_id, token) {
                                                handle_deck_message(deck_msg, &mut write, input_executor.clone(), &config, &active_profile).await?;
                                            } else {
                                                warn!("Paired device was removed; closing connection");
                                                break;
                                            }
                                        } else {
                                            warn!("Rejected deck message before pairing");
                                            let _ = write.send(WsMessage::Close(None)).await;
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to decode Protobuf message: {}", e);
                                        let _ = write.send(WsMessage::Close(None)).await;
                                        break;
                                    }
                                }
                            }
                            WsMessage::Text(text_data) => {
                                if trusted && text_data.len() <= 8192 {
                                    if let Ok(ctrl_msg) = serde_json::from_str::<ControlMessage>(&text_data) {
                                        handle_control_message(ctrl_msg, &mut write, &config, &pairing, &broadcast_tx, &active_profile, &pairing_file).await?;
                                    } else {
                                        write.send(WsMessage::Text(serde_json::to_string(&ControlResponse::Error {
                                            message: "Invalid control message".to_string(),
                                        })?.into())).await?;
                                    }
                                } else {
                                    warn!("Rejected untrusted or oversized control message from: {}", peer_addr);
                                    let _ = write.send(WsMessage::Close(None)).await;
                                    break;
                                }
                            }
                            WsMessage::Ping(payload) => {
                                let _ = write.send(WsMessage::Pong(payload)).await;
                            }
                            WsMessage::Close(_) => {
                                info!("Client {} closed connection", peer_addr);
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => {
                        warn!("Error reading WS: {}", e);
                        break;
                    }
                    None => {
                        break; // Connection closed
                    }
                }
            }
            broadcast_msg = broadcast_rx.recv() => {
                match broadcast_msg {
                    Ok(msg) => {
                        if !trusted && let Some((device_id, token)) = &deck_session {
                            if pairing.lock().unwrap().is_authorized(device_id, token) {
                                send_deck_message(&mut write, msg).await?;
                            } else {
                                warn!("Paired device was removed; closing connection");
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // Channel lagged or closed
                    }
                }
            }
        }
    }

    info!("Connection finished for {}", peer_addr);
    Ok(())
}

async fn handle_control_message<S>(
    msg: ControlMessage,
    write: &mut S,
    config: &Arc<Mutex<AppConfig>>,
    pairing: &Arc<Mutex<PairingStore>>,
    broadcast_tx: &tokio::sync::broadcast::Sender<DeckMessage>,
    active_profile: &Arc<Mutex<String>>,
    pairing_file: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SinkExt<WsMessage> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    match msg {
        ControlMessage::StartPairing => {
            let code = pairing.lock().unwrap().start();
            write
                .send(WsMessage::Text(
                    serde_json::to_string(&ControlResponse::PairingCode { code })?.into(),
                ))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }
        ControlMessage::ListDevices => {
            let device_ids = pairing.lock().unwrap().devices();
            write
                .send(WsMessage::Text(
                    serde_json::to_string(&ControlResponse::Devices { device_ids })?.into(),
                ))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }
        ControlMessage::RevokeDevice { device_id } => {
            let result = pairing.lock().unwrap().revoke(&device_id, pairing_file);
            let response = match result {
                Ok(_) => ControlResponse::Devices {
                    device_ids: pairing.lock().unwrap().devices(),
                },
                Err(error) => ControlResponse::Error {
                    message: error.to_string(),
                },
            };
            write
                .send(WsMessage::Text(serde_json::to_string(&response)?.into()))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }
        ControlMessage::GetProfile { profile } => {
            let resp = {
                let cfg = config.lock().unwrap();
                if let Some(prof) = cfg.profiles.get(&profile) {
                    ControlResponse::Profile {
                        profile: profile.clone(),
                        keys: prof.keys.clone(),
                    }
                } else {
                    ControlResponse::Error {
                        message: "Profile not found".to_string(),
                    }
                }
            };
            let json = serde_json::to_string(&resp)?;
            write
                .send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }
        ControlMessage::SaveProfile { profile, keys } => {
            let candidate = crate::config::Profile {
                name: profile.clone(),
                keys,
            };
            let (saved, layout_msg) = if candidate.validate() {
                let mut cfg = config.lock().unwrap();
                if cfg.profiles.contains_key(&profile) {
                    let previous = cfg.profiles.insert(profile.clone(), candidate.clone());
                    match cfg.save() {
                        Ok(()) => {
                            let current_active = active_profile.lock().unwrap().clone();
                            let layout = (current_active == profile)
                                .then(|| create_layout_message(&profile, &candidate));
                            (true, layout)
                        }
                        Err(error) => {
                            warn!("Could not persist profile: {}", error);
                            if let Some(previous) = previous {
                                cfg.profiles.insert(profile.clone(), previous);
                            }
                            (false, None)
                        }
                    }
                } else {
                    (false, None)
                }
            } else {
                (false, None)
            };

            if saved {
                let resp = ControlResponse::Saved {
                    profile: profile.clone(),
                };
                let json = serde_json::to_string(&resp)?;
                write
                    .send(WsMessage::Text(json.into()))
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

                if let Some(msg) = layout_msg {
                    let _ = broadcast_tx.send(msg);
                }
            } else {
                let resp = ControlResponse::Error {
                    message: "Profile not found or invalid".to_string(),
                };
                let json = serde_json::to_string(&resp)?;
                write
                    .send(WsMessage::Text(json.into()))
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            }
        }
    }
    Ok(())
}

async fn handle_deck_message<S>(
    msg: DeckMessage,
    write: &mut S,
    input_executor: Arc<InputExecutor>,
    config: &Arc<Mutex<AppConfig>>,
    active_profile: &Arc<Mutex<String>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SinkExt<WsMessage> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    if msg.timestamp > 0 {
        let latency = current_time_millis() - msg.timestamp;
        info!("Transport latency delta: {} ms", latency);
    }

    match msg.payload {
        Some(deck_message::Payload::HandshakeReq(_)) => {
            warn!("Unexpected handshake in deck dispatch");
        }
        Some(deck_message::Payload::KeyEvent(event)) => {
            info!(
                "Key Event: slot={}, type={}",
                event.key_index, event.event_type
            );
            if event.event_type == KeyEventType::Down as i32 && (0..9).contains(&event.key_index) {
                let action = {
                    let current_profile = active_profile.lock().unwrap().clone();
                    let cfg = config.lock().unwrap();
                    if let Some(prof) = cfg.profiles.get(&current_profile) {
                        if let Some(key) = prof.keys.iter().find(|k| k.id == event.key_index as u32)
                        {
                            Some(key.action.clone())
                        } else {
                            warn!(
                                "Key index {} out of bounds for profile {}",
                                event.key_index, current_profile
                            );
                            None
                        }
                    } else {
                        None
                    }
                };
                if let Some(act) = action.filter(|action| !action.is_empty()) {
                    tokio::task::spawn_blocking(move || input_executor.trigger_action(&act));
                }
            }
        }
        Some(deck_message::Payload::Heartbeat(hb)) => {
            let pong = DeckMessage {
                timestamp: current_time_millis(),
                payload: Some(deck_message::Payload::Heartbeat(Heartbeat {
                    ping: hb.ping,
                })),
            };
            send_deck_message(write, pong).await?;
        }
        _ => warn!("Unhandled or unsupported message payload"),
    }
    Ok(())
}

async fn handle_handshake<S>(
    req: &HandshakeRequest,
    write: &mut S,
    config: &Arc<Mutex<AppConfig>>,
    pairing: &Arc<Mutex<PairingStore>>,
    active_profile: &Arc<Mutex<String>>,
    pairing_file: &std::path::Path,
) -> Result<Option<(String, String)>, Box<dyn std::error::Error>>
where
    S: SinkExt<WsMessage> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    info!("Handshake received for device {}", req.device_id);
    let authorization = {
        let mut store = pairing.lock().unwrap();
        if req.auth_token.len() == 6 && req.auth_token.bytes().all(|digit| digit.is_ascii_digit()) {
            store.pair(&req.auth_token, &req.device_id, pairing_file)
        } else if store.is_authorized(&req.device_id, &req.auth_token) {
            Ok(Some(req.auth_token.clone()))
        } else {
            Ok(None)
        }
    };
    let token = match authorization {
        Ok(token) => token,
        Err(error) => {
            warn!("Could not save paired device: {}", error);
            None
        }
    };
    let current_profile = active_profile.lock().unwrap().clone();

    let response = DeckMessage {
        timestamp: current_time_millis(),
        payload: Some(deck_message::Payload::HandshakeRes(HandshakeResponse {
            success: token.is_some(),
            server_version: "0.1.0".to_string(),
            message: if token.is_some() {
                "StreamDeck Host Ready"
            } else {
                "Pair this phone in the desktop configurator first"
            }
            .to_string(),
            active_profile_id: current_profile.clone(),
            active_page_id: "main".to_string(),
            session_token: token.clone().unwrap_or_default(),
        })),
    };
    send_deck_message(write, response).await?;
    let Some(token) = token else {
        return Ok(None);
    };

    let layout = {
        let cfg = config.lock().unwrap();
        cfg.profiles
            .get(&current_profile)
            .map(|prof| create_layout_message(&current_profile, prof))
    };

    if let Some(layout_msg) = layout {
        send_deck_message(write, layout_msg).await?;
    }

    Ok(Some((req.device_id.clone(), token)))
}

async fn send_deck_message<S>(
    write: &mut S,
    msg: DeckMessage,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SinkExt<WsMessage> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let mut buf = Vec::new();
    msg.encode(&mut buf)?;
    write
        .send(WsMessage::Binary(buf.into()))
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(())
}

#[cfg(test)]
#[path = "server_test.rs"]
mod tests;
