//! Discord `/science` playtime injection.
//!
//! Ports the technique used by Badge-MAX (`spoofer.py`): post `launch_game`
//! and `running_game_heartbeat` analytics events to `POST /api/v9/science`,
//! the endpoint the official client uses to credit Game Time and Games
//! Played.
//!
//! This file is self-contained. It owns its own HTTP client, its own state
//! and its own commands; nothing else in the app had to change to add it.
//!
//! Deltas from the Python original, all deliberate:
//!
//! * Super properties come from the app's own manager, so the header matches
//!   the rest of the app's traffic instead of a third-party host.
//! * The token is never written to disk. Only the analytics token is cached,
//!   in memory, for 12 hours.
//! * `event_sequence_number` is monotonic for the whole run, not per batch.
//! * A 429 is honoured once per batch (reading `retry_after`) instead of
//!   being swallowed.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use once_cell::sync::Lazy;
use reqwest::header::{ACCEPT, AUTHORIZATION, ORIGIN, REFERER, USER_AGENT};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::AppState;

const SCIENCE_URL: &str = "https://discord.com/api/v9/science";
const ME_URL: &str = "https://discord.com/api/v9/users/@me?with_analytics_token=true";
const DETECTABLE_URL: &str = "https://discord.com/api/v9/applications/detectable";
const GAMES_CDN_URL: &str = "https://cdn.discordapp.com/detectables/games.json";

/// Real clients post one batch at a time and pause between batches.
const BATCH_SIZE: usize = 50;
const BATCH_DELAY: Duration = Duration::from_millis(300);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
/// The analytics token is session-scoped; re-fetch once it ages out.
const ANALYTICS_TTL_MS: i64 = 12 * 60 * 60 * 1000;
/// Upper bound for a single run, in hours. Discord validates elapsed time
/// itself, so a larger number buys nothing but attention.
const MAX_HOURS: f64 = 10_000.0;
const BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) discord/1.0.9260 Chrome/148.0.7778.280 Electron/42.11.8 Safari/537.36";

static RUNNING: AtomicBool = AtomicBool::new(false);
static CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Default, Clone, Serialize)]
pub struct SciencePlaytimeStatus {
    pub running: bool,
    pub total: usize,
    pub processed: usize,
    pub failed: usize,
    pub hours: f64,
    pub started_at_ms: Option<i64>,
    pub finished_at_ms: Option<i64>,
    pub last_server_code: Option<u16>,
    pub last_error: Option<String>,
}

static STATUS: Lazy<Mutex<SciencePlaytimeStatus>> =
    Lazy::new(|| Mutex::new(SciencePlaytimeStatus::default()));

struct AnalyticsCache {
    analytics_token: String,
    fetched_at_ms: i64,
}

static ANALYTICS: Lazy<Mutex<Option<AnalyticsCache>>> = Lazy::new(|| Mutex::new(None));

/// Clears the running flag when a run ends, however it ends.
struct RunGuard;

impl Drop for RunGuard {
    fn drop(&mut self) {
        with_status(|status| {
            status.running = false;
            status.finished_at_ms = Some(now_ms());
        });
    }
}

enum RunError {
    Cancelled,
    Failed(String),
}

#[derive(Debug, Clone)]
struct ScienceGame {
    id: String,
    name: String,
    exe: String,
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

fn with_status(update: impl FnOnce(&mut SciencePlaytimeStatus)) {
    let mut status = STATUS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    update(&mut status);
}

/// Kick off a run. Returns as soon as the run is spawned; progress is read
/// back through `get_science_playtime_status`.
#[tauri::command]
pub async fn start_science_playtime(
    state: State<'_, AppState>,
    hours: f64,
    games_limit: Option<usize>,
) -> Result<(), String> {
    let token = {
        let guard = state
            .client
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard
            .as_ref()
            .ok_or_else(|| "Not authenticated".to_string())?
            .get_token()
            .to_owned()
    };

    if RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("A science playtime run is already active".into());
    }

    let hours = hours.clamp(0.0, MAX_HOURS);
    CANCELLED.store(false, Ordering::SeqCst);
    with_status(|status| {
        *status = SciencePlaytimeStatus {
            running: true,
            hours,
            started_at_ms: Some(now_ms()),
            ..Default::default()
        };
    });

    tokio::spawn(async move {
        let _guard = RunGuard;
        match run_science_playtime(token, hours, games_limit).await {
            Ok(()) | Err(RunError::Cancelled) => {}
            Err(RunError::Failed(error)) => {
                with_status(|status| status.last_error = Some(error));
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_science_playtime_status() -> SciencePlaytimeStatus {
    STATUS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// Ask the running task to stop at the next batch boundary. Returns true
/// when a stop was requested.
#[tauri::command]
pub fn cancel_science_playtime() -> bool {
    let was_running = RUNNING.load(Ordering::SeqCst);
    CANCELLED.store(true, Ordering::SeqCst);
    was_running
}

async fn run_science_playtime(
    token: String,
    hours: f64,
    games_limit: Option<usize>,
) -> Result<(), RunError> {
    let http = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| RunError::Failed(error.to_string()))?;

    let analytics_token = analytics_token(&http, &token)
        .await
        .map_err(RunError::Failed)?;

    let games = fetch_catalog(&http, &token)
        .await
        .map_err(RunError::Failed)?;
    if games.is_empty() {
        return Err(RunError::Failed(
            "No detectable games with a win32 executable were found".into(),
        ));
    }

    let games: Vec<ScienceGame> = match games_limit {
        Some(limit) if limit > 0 => games.into_iter().take(limit).collect(),
        _ => games,
    };

    let duration_ms = (hours * 3_600_000.0).round() as i64;
    let launch_id = uuid::Uuid::new_v4().to_string();
    let launch_signature = uuid::Uuid::new_v4().to_string();
    let heartbeat_session = uuid::Uuid::new_v4().to_string();
    let super_properties =
        super_properties_header(&launch_id, &launch_signature, &heartbeat_session);

    let (locale, timezone) = {
        let manager = crate::SUPER_PROPERTIES_MANAGER
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let profile = manager.get_header_profile();
        (profile.locale.clone(), profile.timezone.clone())
    };

    with_status(|status| status.total = games.len());

    let mut sequence: u64 = 0;
    let mut processed: usize = 0;
    let mut failed: usize = 0;

    for chunk in games.chunks(BATCH_SIZE) {
        if CANCELLED.load(Ordering::SeqCst) {
            return Err(RunError::Cancelled);
        }

        let mut events = Vec::with_capacity(chunk.len() * 3);
        for game in chunk {
            let (batch, next) = build_events(
                game,
                duration_ms,
                &heartbeat_session,
                &launch_signature,
                sequence,
            );
            sequence = next;
            events.extend(batch);
        }

        let mut attempts = 0u8;
        loop {
            let (code, body) = match post_science(
                &http,
                &analytics_token,
                &super_properties,
                &locale,
                &timezone,
                &events,
            )
            .await
            {
                Ok(response) => response,
                Err(error) => {
                    with_status(|status| status.last_error = Some(error.clone()));
                    return Err(RunError::Failed(error));
                }
            };

            match code {
                204 => break,
                429 if attempts == 0 => {
                    attempts += 1;
                    if !sleep_cancellable(retry_delay(&body)).await {
                        return Err(RunError::Cancelled);
                    }
                    continue;
                }
                401 | 403 => {
                    with_status(|status| status.last_server_code = Some(code));
                    return Err(RunError::Failed(format!(
                        "Discord rejected the session (HTTP {code})"
                    )));
                }
                _ => {
                    with_status(|status| status.last_server_code = Some(code));
                    failed += chunk.len();
                    break;
                }
            }
        }

        processed += chunk.len();
        with_status(|status| {
            status.processed = processed;
            status.failed = failed;
        });

        if !sleep_cancellable(BATCH_DELAY).await {
            return Err(RunError::Cancelled);
        }
    }

    Ok(())
}

/// Build the three events the official client emits for one play session:
/// an initial heartbeat, the launch record, and a final heartbeat carrying
/// the credited duration.
fn build_events(
    game: &ScienceGame,
    duration_ms: i64,
    heartbeat_session: &str,
    launch_signature: &str,
    mut sequence: u64,
) -> (Vec<Value>, u64) {
    let now = now_ms();
    let start = now.saturating_sub(duration_ms);
    let game_session = uuid::Uuid::new_v4().to_string();

    sequence += 1;
    let initial = json!({
        "type": "running_game_heartbeat",
        "properties": {
            "client_track_timestamp": start,
            "client_heartbeat_session_id": heartbeat_session,
            "event_sequence_number": sequence,
            "game_id": game.id,
            "game_name": game.name,
            "game_executable": game.exe,
            "game_detection_enabled": true,
            "initial_heartbeat": true,
            "final_heartbeat": false,
            "game_session_id": game_session,
            "duration_tracked_ms": 0,
            "launch_signature": launch_signature,
            "client_app_state": "focused",
            "client_send_timestamp": start
        }
    });

    sequence += 1;
    let launch = json!({
        "type": "launch_game",
        "properties": {
            "client_track_timestamp": now,
            "client_heartbeat_session_id": heartbeat_session,
            "event_sequence_number": sequence,
            "game": game.name,
            "game_id": game.id,
            "verified": true,
            "elevated": false,
            "is_launcher": false,
            "game_platform": "desktop",
            "detection_method": "verified_game",
            "is_overlay_enabled": false,
            "is_overlay_game_enabled": true,
            "is_overlay_game_source": "OOP_DEFAULT_DATABASE",
            "fullscreen_type": "UNKNOWN",
            "hardware_display_count": 1,
            "overlay_method": "Disabled",
            "activity_status_enabled": true,
            "activity_status_shared_guilds": [],
            "current_user_status": "online",
            "game_detection_enabled": true,
            "executable_path": game.exe,
            "rendered_locale": "en-GB",
            "launch_signature": launch_signature,
            "client_app_state": "focused",
            "client_send_timestamp": now
        }
    });

    sequence += 1;
    let final_heartbeat = json!({
        "type": "running_game_heartbeat",
        "properties": {
            "client_track_timestamp": now,
            "client_heartbeat_session_id": heartbeat_session,
            "event_sequence_number": sequence,
            "game_id": game.id,
            "game_name": game.name,
            "game_executable": game.exe,
            "game_detection_enabled": true,
            "initial_heartbeat": false,
            "final_heartbeat": true,
            "game_session_id": game_session,
            "duration_tracked_ms": duration_ms,
            "launch_signature": launch_signature,
            "client_app_state": "focused",
            "client_send_timestamp": now
        }
    });

    (vec![initial, launch, final_heartbeat], sequence)
}

/// The app's own super properties, with this run's launch identifiers
/// stamped in so the header agrees with the events it ships alongside.
fn super_properties_header(
    launch_id: &str,
    launch_signature: &str,
    heartbeat_session: &str,
) -> String {
    let base64_value = {
        let manager = crate::SUPER_PROPERTIES_MANAGER
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        manager.get_super_properties_base64()
    };

    let mut properties: Value = BASE64
        .decode(base64_value.as_bytes())
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        .unwrap_or_else(|| json!({}));

    if let Value::Object(map) = &mut properties {
        map.insert("client_launch_id".to_string(), json!(launch_id));
        map.insert("launch_signature".to_string(), json!(launch_signature));
        map.insert(
            "client_heartbeat_session_id".to_string(),
            json!(heartbeat_session),
        );
    }

    match serde_json::to_vec(&properties) {
        Ok(raw) => BASE64.encode(raw),
        Err(_) => base64_value,
    }
}

async fn post_science(
    http: &reqwest::Client,
    analytics_token: &str,
    super_properties: &str,
    locale: &str,
    timezone: &str,
    events: &[Value],
) -> Result<(u16, Value), String> {
    let response = http
        .post(SCIENCE_URL)
        .header(AUTHORIZATION, analytics_token)
        .header(ACCEPT, "*/*")
        .header(ORIGIN, "https://discord.com")
        .header(REFERER, "https://discord.com/channels/@me")
        .header(USER_AGENT, BROWSER_USER_AGENT)
        .header("x-super-properties", super_properties)
        .header("x-discord-locale", locale)
        .header("x-discord-timezone", timezone)
        .json(&json!({ "token": analytics_token, "events": events }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let code = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    let parsed = serde_json::from_str::<Value>(&body).unwrap_or(Value::Null);
    Ok((code, parsed))
}

/// `retry_after` is the one field on a 429 worth reading; everything else
/// falls back to a fixed pause.
fn retry_delay(body: &Value) -> Duration {
    let seconds = body
        .get("retry_after")
        .and_then(Value::as_f64)
        .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
        .unwrap_or(1.5);

    Duration::from_millis((seconds * 1_000.0).round().clamp(0.0, 5_000.0) as u64)
}

/// Sleep in small slices so a cancel lands within 50ms instead of after the
/// whole batch delay. Returns false when cancelled.
async fn sleep_cancellable(duration: Duration) -> bool {
    const STEP: Duration = Duration::from_millis(50);
    let mut remaining = duration;

    while !remaining.is_zero() {
        if CANCELLED.load(Ordering::SeqCst) {
            return false;
        }
        let wait = remaining.min(STEP);
        tokio::time::sleep(wait).await;
        remaining -= wait;
    }

    !CANCELLED.load(Ordering::SeqCst)
}

async fn analytics_token(http: &reqwest::Client, token: &str) -> Result<String, String> {
    if let Some(cached) = cached_analytics_token() {
        return Ok(cached);
    }

    let response = http
        .get(ME_URL)
        .header(AUTHORIZATION, token)
        .header(USER_AGENT, BROWSER_USER_AGENT)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let parsed = serde_json::from_str::<Value>(&body).unwrap_or(Value::Null);

    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err("Discord rejected the session (bad or expired token)".into());
    }

    let analytics_token = parsed
        .get("analytics_token")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "Discord did not return an analytics token (HTTP {})",
                status.as_u16()
            )
        })?;

    *ANALYTICS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(AnalyticsCache {
        analytics_token: analytics_token.clone(),
        fetched_at_ms: now_ms(),
    });

    Ok(analytics_token)
}

fn cached_analytics_token() -> Option<String> {
    let cache = ANALYTICS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .as_ref()
        .filter(|entry| now_ms() - entry.fetched_at_ms < ANALYTICS_TTL_MS)
        .map(|entry| entry.analytics_token.clone())
}

/// The game catalog comes from Discord's public CDN first (no auth, no rate
/// budget spent); the authenticated endpoint is the fallback.
async fn fetch_catalog(http: &reqwest::Client, token: &str) -> Result<Vec<ScienceGame>, String> {
    let mut games = fetch_json(http, GAMES_CDN_URL, None)
        .await
        .map(|value| parse_games(&value));

    if games.as_deref().map_or(true, |list| list.is_empty()) {
        games = fetch_json(http, DETECTABLE_URL, Some(token))
            .await
            .map(|value| parse_games(&value));
    }

    games
}

async fn fetch_json(
    http: &reqwest::Client,
    url: &str,
    token: Option<&str>,
) -> Result<Value, String> {
    let mut request = http.get(url).header(USER_AGENT, BROWSER_USER_AGENT);
    if let Some(token) = token {
        request = request.header(AUTHORIZATION, token);
    }

    let response = request.send().await.map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "GET {url} failed (HTTP {})",
            response.status().as_u16()
        ));
    }

    let body = response.text().await.map_err(|error| error.to_string())?;
    serde_json::from_str::<Value>(&body).map_err(|error| error.to_string())
}

/// Keep only numeric, deduplicated ids with a win32 executable - that is the
/// shape `running_game_heartbeat` can name.
fn parse_games(value: &Value) -> Vec<ScienceGame> {
    let Some(list) = value.as_array() else {
        return Vec::new();
    };

    let mut games = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for entry in list {
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        if id.is_empty() || !id.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }
        if !seen.insert(id.clone()) {
            continue;
        }

        let exe = entry
            .get("executables")
            .and_then(Value::as_array)
            .and_then(|executables| {
                executables.iter().find(|executable| {
                    executable.get("os").and_then(Value::as_str) == Some("win32")
                        && executable
                            .get("name")
                            .and_then(Value::as_str)
                            .is_some_and(|name| !name.is_empty())
                })
            })
            .and_then(|executable| executable.get("name").and_then(Value::as_str));

        let Some(exe) = exe else {
            continue;
        };

        games.push(ScienceGame {
            id,
            name: entry
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unknown")
                .to_string(),
            exe: exe.to_owned(),
        });
    }

    games
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_games_keeps_only_win32_executables() {
        let catalog = json!([
            {
                "id": "356875632244871179",
                "name": "Hades II",
                "executables": [{ "os": "win32", "name": "Hades2.exe" }, { "os": "macos", "name": "Hades2" }]
            },
            {
                "id": "not-a-number",
                "name": "Skipped",
                "executables": [{ "os": "win32", "name": "skipped.exe" }]
            },
            {
                "id": "415000000000000001",
                "name": "No win32",
                "executables": [{ "os": "linux", "name": "game.bin" }]
            },
            {
                "id": "356875632244871179",
                "name": "Duplicate",
                "executables": [{ "os": "win32", "name": "dup.exe" }]
            }
        ]);

        let games = parse_games(&catalog);

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "356875632244871179");
        assert_eq!(games[0].name, "Hades II");
        assert_eq!(games[0].exe, "Hades2.exe");
    }

    #[test]
    fn parse_games_tolerates_non_array_payloads() {
        assert!(parse_games(&json!({ "error": "rate limited" })).is_empty());
        assert!(parse_games(&json!(null)).is_empty());
    }

    #[test]
    fn build_events_is_three_events_with_a_running_sequence() {
        let game = ScienceGame {
            id: "1".into(),
            name: "Game".into(),
            exe: "game.exe".into(),
        };

        let (events, sequence) = build_events(&game, 3_600_000, "hb", "sig", 0);

        assert_eq!(events.len(), 3);
        assert_eq!(sequence, 3);

        let initial = &events[0];
        assert_eq!(initial["properties"]["event_sequence_number"], 1);
        assert_eq!(initial["properties"]["duration_tracked_ms"], 0);
        assert_eq!(initial["properties"]["initial_heartbeat"], true);

        let launch = &events[1];
        assert_eq!(launch["properties"]["event_sequence_number"], 2);
        assert_eq!(launch["type"], "launch_game");

        let final_heartbeat = &events[2];
        assert_eq!(final_heartbeat["properties"]["event_sequence_number"], 3);
        assert_eq!(final_heartbeat["properties"]["final_heartbeat"], true);
        assert_eq!(
            final_heartbeat["properties"]["duration_tracked_ms"],
            3_600_000
        );

        // The initial and final heartbeat share one game session.
        assert_eq!(
            initial["properties"]["game_session_id"],
            final_heartbeat["properties"]["game_session_id"]
        );
    }

    #[test]
    fn super_properties_header_stamps_run_identifiers() {
        let encoded = super_properties_header("launch-id", "signature", "heartbeat-session");
        let decoded = BASE64
            .decode(encoded.as_bytes())
            .expect("header must be base64");
        let properties: Value =
            serde_json::from_slice(&decoded).expect("header must decode to JSON");

        assert_eq!(properties["client_launch_id"], "launch-id");
        assert_eq!(properties["launch_signature"], "signature");
        assert_eq!(
            properties["client_heartbeat_session_id"],
            "heartbeat-session"
        );
    }

    #[test]
    fn retry_delay_reads_retry_after_and_bounds_it() {
        assert_eq!(
            retry_delay(&json!({ "retry_after": 2.0 })),
            Duration::from_millis(2_000)
        );
        assert_eq!(
            retry_delay(&json!({ "retry_after": 120.0 })),
            Duration::from_millis(5_000)
        );
        assert_eq!(retry_delay(&json!({})), Duration::from_millis(1_500));
        assert_eq!(retry_delay(&Value::Null), Duration::from_millis(1_500));
    }

    #[tokio::test]
    async fn sleep_cancellable_returns_false_when_cancelled() {
        CANCELLED.store(true, Ordering::SeqCst);
        let result = sleep_cancellable(Duration::from_millis(200)).await;
        CANCELLED.store(false, Ordering::SeqCst);

        assert!(!result);
    }
}
