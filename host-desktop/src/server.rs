// [LINE BUDGET AUDIT] 190/250
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use prost::Message;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{error, info, warn};

use crate::input::InputExecutor;
use crate::layout::{create_default_layout, current_time_millis};
use crate::protocol::streamdeck::{
    deck_message, DeckMessage, HandshakeRequest, HandshakeResponse, Heartbeat, KeyEventType,
};
use crate::audio::AudioController;
use crate::profiles::ProfileWatcher;

pub struct DeckServer {
    #[allow(dead_code)]
    addr: String,
    input_executor: Arc<InputExecutor>,
    #[allow(dead_code)]
    audio: Arc<Mutex<AudioController>>,
}

impl DeckServer {
    pub fn new(addr: impl Into<String>) -> Self {
        let audio = AudioController::new().unwrap_or_else(|_| AudioController::stub());
        let input_executor = Arc::new(InputExecutor::new(audio.clone()));
        Self {
            addr: addr.into(),
            input_executor,
            audio,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        ProfileWatcher::spawn(tx);
        tokio::spawn(async move {
            while let Some(profile) = rx.recv().await {
                info!("Profile changed to: {}", profile);
            }
        });

        let bind_addrs = ["0.0.0.0:4455", "[::1]:4455"];
        let mut listeners = Vec::new();

        for addr in bind_addrs {
            match TcpListener::bind(addr).await {
                Ok(l) => {
                    info!("StreamDeck Host listening on ws://{}", addr);
                    listeners.push(l);
                }
                Err(e) => warn!("Could not bind to {}: {}", addr, e),
            }
        }

        if listeners.is_empty() {
            return Err("Failed to bind any network listener on port 4455".into());
        }

        let mut tasks = Vec::new();
        for listener in listeners {
            let executor = Arc::clone(&self.input_executor);
            tasks.push(tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, peer_addr)) => {
                            let exec = Arc::clone(&executor);
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, peer_addr, exec).await {
                                    warn!("Connection error with {}: {}", peer_addr, e);
                                }
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

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    input_executor: Arc<InputExecutor>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    info!("Accepted WebSocket connection from {}", peer_addr);
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg_result) = read.next().await {
        let msg = msg_result?;
        match msg {
            WsMessage::Binary(bin_data) => {
                match DeckMessage::decode(bin_data.as_ref()) {
                    Ok(deck_msg) => {
                        handle_deck_message(deck_msg, &mut write, &input_executor).await?;
                    }
                    Err(e) => warn!("Failed to decode Protobuf message: {}", e),
                }
            }
            WsMessage::Ping(payload) => write.send(WsMessage::Pong(payload)).await?,
            WsMessage::Close(_) => {
                info!("Client {} closed connection", peer_addr);
                break;
            }
            _ => {}
        }
    }

    info!("Connection finished for {}", peer_addr);
    Ok(())
}

async fn handle_deck_message<S>(
    msg: DeckMessage,
    write: &mut S,
    input_executor: &InputExecutor,
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
        Some(deck_message::Payload::HandshakeReq(req)) => {
            handle_handshake(req, write).await?;
        }
        Some(deck_message::Payload::KeyEvent(event)) => {
            info!("Key Event: slot={}, type={}", event.key_index, event.event_type);
            if event.event_type == KeyEventType::Down as i32 {
                input_executor.trigger_key_index(event.key_index);
            }
        }
        Some(deck_message::Payload::Heartbeat(hb)) => {
            let pong = DeckMessage {
                timestamp: current_time_millis(),
                payload: Some(deck_message::Payload::Heartbeat(Heartbeat { ping: hb.ping })),
            };
            send_deck_message(write, pong).await?;
        }
        _ => warn!("Unhandled or unsupported message payload"),
    }
    Ok(())
}

async fn handle_handshake<S>(
    req: HandshakeRequest,
    write: &mut S,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SinkExt<WsMessage> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    info!("Handshake from: {} ({})", req.device_id, req.device_name);

    let response = DeckMessage {
        timestamp: current_time_millis(),
        payload: Some(deck_message::Payload::HandshakeRes(HandshakeResponse {
            success: true,
            server_version: "0.1.0".to_string(),
            message: "StreamDeck Host Ready".to_string(),
            active_profile_id: "default".to_string(),
            active_page_id: "main".to_string(),
            session_token: "session-ok".to_string(),
        })),
    };
    send_deck_message(write, response).await?;
    send_deck_message(write, create_default_layout()).await?;
    Ok(())
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
    write.send(WsMessage::Binary(buf.into())).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(())
}
