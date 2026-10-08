use super::*;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

type ClientSocket = tokio_tungstenite::WebSocketStream<TcpStream>;
type SharedConfig = Arc<Mutex<AppConfig>>;
type SharedPairing = Arc<Mutex<PairingStore>>;
type SharedProfile = Arc<Mutex<String>>;
type Broadcast = tokio::sync::broadcast::Sender<DeckMessage>;

async fn connect(
    listener: &TcpListener,
    origin: Option<&str>,
    state: &(
        Arc<InputExecutor>,
        SharedConfig,
        SharedPairing,
        SharedProfile,
        Broadcast,
    ),
    file: &std::path::Path,
) -> ClientSocket {
    let address = listener.local_addr().unwrap();
    let mut request = format!("ws://{address}").into_client_request().unwrap();
    if let Some(origin) = origin {
        request
            .headers_mut()
            .insert("origin", origin.parse().unwrap());
        request.headers_mut().insert(
            "authorization",
            "Bearer test-control-secret".parse().unwrap(),
        );
    }
    let client = TcpStream::connect(address).await.unwrap();
    let (server, peer) = listener.accept().await.unwrap();
    let (executor, config, pairing, profile, tx) = state.clone();
    let file = file.to_path_buf();
    let rx = tx.subscribe();
    let control_secret = origin.map(|_| "test-control-secret".to_string());
    tokio::spawn(async move {
        handle_connection(
            server,
            peer,
            executor,
            config,
            pairing,
            profile,
            tx,
            rx,
            file,
            control_secret,
            None,
        )
        .await
        .unwrap();
    });
    client_async(request, client).await.unwrap().0
}

#[tokio::test]
async fn control_rejects_missing_or_wrong_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("paired-devices.json");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let state = (
        Arc::new(InputExecutor::new(AudioController::stub())),
        Arc::new(Mutex::new(AppConfig::default_config())),
        Arc::new(Mutex::new(PairingStore::default())),
        Arc::new(Mutex::new("Default".to_string())),
        tokio::sync::broadcast::channel(16).0,
    );
    for (origin, credential) in [
        ("tauri://localhost", None),
        ("tauri://localhost", Some("Bearer wrong")),
        (
            "https://untrusted.example",
            Some("Bearer test-control-secret"),
        ),
    ] {
        let address = listener.local_addr().unwrap();
        let mut request = format!("ws://{address}").into_client_request().unwrap();
        request
            .headers_mut()
            .insert("origin", origin.parse().unwrap());
        if let Some(credential) = credential {
            request
                .headers_mut()
                .insert("authorization", credential.parse().unwrap());
        }
        let client = TcpStream::connect(address).await.unwrap();
        let (server, peer) = listener.accept().await.unwrap();
        let (executor, config, pairing, profile, tx) = state.clone();
        let rx = tx.subscribe();
        let file = file.clone();
        tokio::spawn(async move {
            let _ = handle_connection(
                server,
                peer,
                executor,
                config,
                pairing,
                profile,
                tx,
                rx,
                file,
                Some("test-control-secret".to_string()),
                None,
            )
            .await;
        });
        assert!(client_async(request, client).await.is_err());
    }
    let mut gui = connect(&listener, Some("tauri://localhost"), &state, &file).await;
    gui.send(WsMessage::Text(r#"{"type":"list_devices"}"#.into()))
        .await
        .unwrap();
    assert!(gui.next().await.unwrap().unwrap().is_text());
}

#[tokio::test]
async fn oversized_phone_frame_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("paired-devices.json");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let state = (
        Arc::new(InputExecutor::new(AudioController::stub())),
        Arc::new(Mutex::new(AppConfig::default_config())),
        Arc::new(Mutex::new(PairingStore::default())),
        Arc::new(Mutex::new("Default".to_string())),
        tokio::sync::broadcast::channel(16).0,
    );
    let mut phone = connect(&listener, None, &state, &file).await;
    phone
        .send(WsMessage::Binary(vec![0; 65_537].into()))
        .await
        .unwrap();
    assert!(matches!(
        timeout(Duration::from_secs(3), phone.next()).await.unwrap(),
        Some(Ok(WsMessage::Close(_))) | None | Some(Err(_))
    ));
}

#[tokio::test]
async fn malformed_phone_frame_closes_socket() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("paired-devices.json");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let state = (
        Arc::new(InputExecutor::new(AudioController::stub())),
        Arc::new(Mutex::new(AppConfig::default_config())),
        Arc::new(Mutex::new(PairingStore::default())),
        Arc::new(Mutex::new("Default".to_string())),
        tokio::sync::broadcast::channel(16).0,
    );
    let mut phone = connect(&listener, None, &state, &file).await;
    phone
        .send(WsMessage::Binary(vec![0xff, 0xff].into()))
        .await
        .unwrap();
    assert!(matches!(
        timeout(Duration::from_secs(3), phone.next()).await.unwrap(),
        Some(Ok(WsMessage::Close(_))) | None
    ));
}

#[tokio::test]
async fn repeated_handshake_closes_phone_socket() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("paired-devices.json");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let state = (
        Arc::new(InputExecutor::new(AudioController::stub())),
        Arc::new(Mutex::new(AppConfig::default_config())),
        Arc::new(Mutex::new(PairingStore::default())),
        Arc::new(Mutex::new("Default".to_string())),
        tokio::sync::broadcast::channel(16).0,
    );
    let mut gui = connect(&listener, Some("tauri://localhost"), &state, &file).await;
    gui.send(WsMessage::Text(r#"{"type":"start_pairing"}"#.into()))
        .await
        .unwrap();
    let response = gui.next().await.unwrap().unwrap().into_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(&response).unwrap();
    let code = value["code"].as_str().unwrap();

    let mut phone = connect(&listener, None, &state, &file).await;
    send_deck(&mut phone, handshake(code)).await;
    assert!(handshake_response(read_deck(&mut phone).await).success);
    let _ = read_deck(&mut phone).await;
    send_deck(&mut phone, handshake(code)).await;
    assert!(matches!(
        timeout(Duration::from_secs(3), phone.next()).await.unwrap(),
        Some(Ok(WsMessage::Close(_))) | None
    ));
}

async fn send_deck(ws: &mut ClientSocket, message: DeckMessage) {
    let mut bytes = Vec::new();
    message.encode(&mut bytes).unwrap();
    ws.send(WsMessage::Binary(bytes.into())).await.unwrap();
}

async fn read_deck(ws: &mut ClientSocket) -> DeckMessage {
    let message = timeout(Duration::from_secs(3), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let WsMessage::Binary(bytes) = message else {
        panic!("expected binary response: {message:?}");
    };
    DeckMessage::decode(bytes.as_ref()).unwrap()
}

fn handshake_response(message: DeckMessage) -> HandshakeResponse {
    let Some(deck_message::Payload::HandshakeRes(response)) = message.payload else {
        panic!("expected handshake response");
    };
    response
}

fn handshake(token: &str) -> DeckMessage {
    DeckMessage {
        timestamp: 0,
        payload: Some(deck_message::Payload::HandshakeReq(HandshakeRequest {
            device_id: "test-phone".into(),
            device_name: "Phone".into(),
            client_version: "0.1".into(),
            screen_width_dp: 400,
            screen_height_dp: 800,
            auth_token: token.into(),
        })),
    }
}

#[tokio::test]
async fn pairing_blocks_unknown_phones_and_revocation_closes_active_session() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("paired-devices.json");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let state = (
        Arc::new(InputExecutor::new(AudioController::stub())),
        Arc::new(Mutex::new(AppConfig::default_config())),
        Arc::new(Mutex::new(PairingStore::default())),
        Arc::new(Mutex::new("Default".to_string())),
        tokio::sync::broadcast::channel(16).0,
    );
    let mut gui = connect(&listener, Some("tauri://localhost"), &state, &file).await;
    let mut phone = connect(&listener, None, &state, &file).await;

    send_deck(&mut phone, handshake("")).await;
    assert!(!handshake_response(read_deck(&mut phone).await).success);
    assert!(matches!(
        phone.next().await,
        Some(Ok(WsMessage::Close(_))) | None
    ));
    let mut phone = connect(&listener, None, &state, &file).await;

    gui.send(WsMessage::Text(r#"{"type":"start_pairing"}"#.into()))
        .await
        .unwrap();
    let message = gui.next().await.unwrap().unwrap().into_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(&message).unwrap();
    let code = value["code"].as_str().unwrap();
    assert!(!code.is_empty());

    send_deck(&mut phone, handshake(code)).await;
    let response = handshake_response(read_deck(&mut phone).await);
    assert!(response.success);
    let credential = response.session_token;
    assert!(!credential.is_empty());
    assert!(matches!(
        read_deck(&mut phone).await.payload,
        Some(deck_message::Payload::LayoutUpdate(_))
    ));
    assert!(
        PairingStore::load(&file)
            .unwrap()
            .is_authorized("test-phone", &credential)
    );

    let mut returning = connect(&listener, None, &state, &file).await;
    send_deck(&mut returning, handshake(&credential)).await;
    assert!(handshake_response(read_deck(&mut returning).await).success);
    let _ = read_deck(&mut returning).await;

    gui.send(WsMessage::Text(
        r#"{"type":"revoke_device","device_id":"test-phone"}"#.into(),
    ))
    .await
    .unwrap();
    let devices = gui.next().await.unwrap().unwrap().into_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(&devices).unwrap();
    assert_eq!(value["device_ids"].as_array().unwrap().len(), 0);
    for socket in [&mut returning, &mut phone] {
        assert!(matches!(
            timeout(Duration::from_secs(3), socket.next())
                .await
                .unwrap(),
            Some(Ok(WsMessage::Close(_))) | None
        ));
    }

    let mut removed = connect(&listener, None, &state, &file).await;
    send_deck(&mut removed, handshake(&credential)).await;
    assert!(!handshake_response(read_deck(&mut removed).await).success);
}
