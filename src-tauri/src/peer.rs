use iroh::{protocol::Router, Endpoint};
use iroh_ping::Ping;
use iroh_tickets::{endpoint::EndpointTicket, Ticket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Represents a connected peer
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerInfo {
    /// The remote endpoint ID (public key, hex-encoded)
    pub node_id: String,
    /// Display name shared by the peer (if any)
    pub username: String,
    /// Whether the peer is currently reachable
    pub online: bool,
    /// Last measured round-trip time in milliseconds
    pub rtt_ms: Option<f64>,
    /// When we last successfully pinged this peer (unix timestamp ms)
    pub last_seen: Option<u64>,
}

/// Payload emitted to the frontend when peer statuses change
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerStatusEvent {
    pub peers: Vec<PeerInfo>,
}

/// Internal state kept while the iroh endpoint is running
struct IrohInner {
    endpoint: Endpoint,
    router: Router,
    ping: Ping,
    /// Known peers: node_id_hex -> PeerInfo
    peers: HashMap<String, PeerInfo>,
}

/// Thread-safe wrapper managed by Tauri
pub struct PeerState {
    inner: Option<IrohInner>,
}

impl PeerState {
    pub fn new() -> Self {
        Self { inner: None }
    }
}

// ─── Tauri Commands ───────────────────────────────────────────────

/// Start the iroh endpoint and router. Returns the local node ID.
#[tauri::command]
pub async fn peer_start(
    app: AppHandle,
    state: tauri::State<'_, Arc<Mutex<PeerState>>>,
) -> Result<String, String> {
    let mut guard = state.lock().await;

    if let Some(ref inner) = guard.inner {
        let node_id = inner.endpoint.id().to_string();
        log::info!("[Peer] Endpoint already running: {}", node_id);
        return Ok(node_id);
    }

    log::info!("[Peer] Starting iroh endpoint...");

    let endpoint = Endpoint::bind()
        .await
        .map_err(|e| format!("Failed to bind endpoint: {e}"))?;

    endpoint.online().await;

    let ping = Ping::new();
    let router = Router::builder(endpoint.clone())
        .accept(iroh_ping::ALPN, ping.clone())
        .spawn();

    let node_id = endpoint.id().to_string();
    log::info!("[Peer] Endpoint online, node_id={}", node_id);

    guard.inner = Some(IrohInner {
        endpoint,
        router,
        ping,
        peers: HashMap::new(),
    });

    // Start the background heartbeat task
    let state_clone = Arc::clone(&*state);
    let app_clone = app.clone();
    tokio::spawn(async move {
        heartbeat_loop(state_clone, app_clone).await;
    });

    Ok(node_id)
}

/// Stop the iroh endpoint and router.
#[tauri::command]
pub async fn peer_stop(
    state: tauri::State<'_, Arc<Mutex<PeerState>>>,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    if let Some(inner) = guard.inner.take() {
        log::info!("[Peer] Shutting down iroh endpoint...");
        inner.router.shutdown().await.map_err(|e| e.to_string())?;
        log::info!("[Peer] Endpoint stopped.");
    }
    Ok(())
}

/// Generate an endpoint ticket that another user can paste to connect.
#[tauri::command]
pub async fn peer_generate_ticket(
    state: tauri::State<'_, Arc<Mutex<PeerState>>>,
) -> Result<String, String> {
    let guard = state.lock().await;
    let inner = guard.inner.as_ref().ok_or("Endpoint not started")?;

    let ticket = EndpointTicket::new(inner.endpoint.addr());
    let ticket_str = Ticket::serialize(&ticket);
    log::info!("[Peer] Generated ticket ({} chars)", ticket_str.len());
    Ok(ticket_str)
}

/// Connect to a remote peer by pasting their ticket. Returns the PeerInfo.
#[tauri::command]
pub async fn peer_connect(
    app: AppHandle,
    state: tauri::State<'_, Arc<Mutex<PeerState>>>,
    ticket_str: String,
    username: String,
) -> Result<PeerInfo, String> {
    let ticket = <EndpointTicket as Ticket>::deserialize(&ticket_str)
        .map_err(|e| format!("Invalid ticket: {e}"))?;

    let remote_addr = ticket.endpoint_addr().clone();
    let remote_node_id = remote_addr.id.to_string();

    log::info!("[Peer] Connecting to peer {}...", remote_node_id);

    // Ping to verify connectivity
    let mut guard = state.lock().await;
    let inner = guard.inner.as_mut().ok_or("Endpoint not started")?;

    let rtt = inner
        .ping
        .ping(&inner.endpoint, remote_addr)
        .await
        .map_err(|e| format!("Ping failed: {e}"))?;

    let rtt_ms = rtt.as_secs_f64() * 1000.0;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let peer = PeerInfo {
        node_id: remote_node_id.clone(),
        username,
        online: true,
        rtt_ms: Some(rtt_ms),
        last_seen: Some(now_ms),
    };

    log::info!(
        "[Peer] Connected to {} (rtt={:.1}ms)",
        remote_node_id,
        rtt_ms
    );

    inner.peers.insert(remote_node_id, peer.clone());

    // Emit updated peer list
    emit_peer_status(&inner.peers, &app);

    Ok(peer)
}

/// Remove a peer from the known list.
#[tauri::command]
pub async fn peer_disconnect(
    app: AppHandle,
    state: tauri::State<'_, Arc<Mutex<PeerState>>>,
    node_id: String,
) -> Result<(), String> {
    let mut guard = state.lock().await;
    let inner = guard.inner.as_mut().ok_or("Endpoint not started")?;
    inner.peers.remove(&node_id);
    log::info!("[Peer] Removed peer {}", node_id);
    emit_peer_status(&inner.peers, &app);
    Ok(())
}

/// Get the list of known peers.
#[tauri::command]
pub async fn peer_list(
    state: tauri::State<'_, Arc<Mutex<PeerState>>>,
) -> Result<Vec<PeerInfo>, String> {
    let guard = state.lock().await;
    let inner = guard.inner.as_ref().ok_or("Endpoint not started")?;
    Ok(inner.peers.values().cloned().collect())
}

// ─── Helpers ──────────────────────────────────────────────────────

fn emit_peer_status(peers: &HashMap<String, PeerInfo>, app: &AppHandle) {
    let event = PeerStatusEvent {
        peers: peers.values().cloned().collect(),
    };
    if let Err(e) = app.emit("peer-status", &event) {
        log::warn!("[Peer] Failed to emit peer-status event: {e}");
    }
}

/// Background loop that pings all known peers every 30 seconds to check liveness.
async fn heartbeat_loop(state: Arc<Mutex<PeerState>>, app: AppHandle) {
    let interval = Duration::from_secs(30);
    loop {
        tokio::time::sleep(interval).await;

        let mut guard = state.lock().await;
        let inner = match guard.inner.as_mut() {
            Some(i) => i,
            None => {
                log::info!("[Peer] Heartbeat: endpoint gone, stopping loop.");
                return;
            }
        };

        let peer_ids: Vec<String> = inner.peers.keys().cloned().collect();
        if peer_ids.is_empty() {
            continue;
        }

        log::debug!("[Peer] Heartbeat: pinging {} peers", peer_ids.len());

        for node_id in &peer_ids {
            // Check if the endpoint has remote info for this peer
            let endpoint_id: iroh::EndpointId = match node_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    log::warn!("[Peer] Heartbeat: invalid node_id {}", node_id);
                    continue;
                }
            };

            // Build addr from endpoint ID – discovery will resolve the rest
            let addr = iroh::EndpointAddr::from(endpoint_id);

            let ping_result = tokio::time::timeout(
                Duration::from_secs(10),
                inner.ping.ping(&inner.endpoint, addr),
            )
            .await;

            if let Some(peer) = inner.peers.get_mut(node_id) {
                match ping_result {
                    Ok(Ok(rtt)) => {
                        peer.online = true;
                        peer.rtt_ms = Some(rtt.as_secs_f64() * 1000.0);
                        peer.last_seen = Some(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                        );
                    }
                    _ => {
                        peer.online = false;
                        log::debug!("[Peer] Heartbeat: peer {} unreachable", node_id);
                    }
                }
            }
        }

        emit_peer_status(&inner.peers, &app);
    }
}
