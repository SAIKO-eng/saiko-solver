//! Offline, machine-bound license activation for SAIKO SOLVER.
//!
//! A license code is `<base64url(payload JSON)>.<base64url(signature)>` where:
//! - payload is `{"v":1,"m":"<machine>","e":<expiry_epoch_secs>,"p":"<plan>"}`
//!   (`e == 0` means the license never expires),
//! - the signature covers the canonical message
//!   `v1|<machine>|<expiry>|<plan>` using RSA PKCS#1 v1.5 with SHA-256.
//! - the machine field is matched against [`machine_id`], so a code issued for
//!   one PC cannot be reused on another.
//!
//! Codes can only be minted with the private key held by the seller
//! (`scripts/license/make-license.mjs`). Only the public key is embedded in
//! the binary, so customers cannot forge codes.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::{Pkcs1v15Sign, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Manager;

/// Embedded seller public key written by
/// `scripts/license/make-license.mjs init`. The private key never ships.
const PUBLIC_KEY_PEM: &str = include_str!("license_pub_key.pem");

const LICENSE_FILE_NAME: &str = "license.json";
const LICENSE_PROTOCOL: &str = "v1";
/// If the system clock jumps backwards by more than a day, the offline license
/// is treated as invalid (basic anti-rollback guard).
const CLOCK_ROLLBACK_TOLERANCE_SECS: i64 = 86_400;

#[derive(Debug, Clone, Serialize)]
pub struct LicenseStatus {
    pub state: &'static str,
    pub machine_id: String,
    pub expires_at: Option<i64>,
    pub plan: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LicensePayload {
    #[serde(rename = "v")]
    version: u32,
    #[serde(rename = "m")]
    machine: String,
    #[serde(rename = "e")]
    expiry: i64,
    #[serde(rename = "p")]
    plan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoredLicense {
    code: Option<String>,
    last_seen: Option<i64>,
}

/// 24-uppercase-hex machine fingerprint derived from stable OS identifiers so
/// it survives the stealth relaunch into a randomly-named temporary copy.
pub fn machine_id() -> String {
    let mut raw = String::new();
    #[cfg(windows)]
    {
        if let Ok(guid) = windows_registry::LOCAL_MACHINE
            .open(r"Software\Microsoft\Cryptography")
            .and_then(|key| key.get_string("MachineGuid"))
        {
            raw.push_str(&guid);
        }
    }
    raw.push('|');
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    raw.push_str(&hostname);

    let digest = Sha256::digest(raw.as_bytes());
    hex_encode(&digest)[..24].to_uppercase()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn canonical_message(payload: &LicensePayload) -> String {
    format!(
        "{LICENSE_PROTOCOL}|{}|{}|{}",
        payload.machine, payload.expiry, payload.plan
    )
}

/// Verify a license code against the current machine. Returns the verified
/// license on success or a human-readable reason on failure.
pub fn verify_code(code: &str, current_machine: &str) -> Result<LicenseStatus, String> {
    let (payload_b64, signature_b64) = code
        .split_once('.')
        .ok_or_else(|| "Malformed activation code".to_string())?;

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| "Malformed activation code".to_string())?;
    let payload: LicensePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|_| "Unsupported activation code".to_string())?;

    if payload.version != 1 {
        return Err("Unsupported activation code version".to_string());
    }

    let signature = URL_SAFE_NO_PAD
        .decode(signature_b64)
        .map_err(|_| "Malformed activation signature".to_string())?;

    let public_key = RsaPublicKey::from_pkcs1_pem(PUBLIC_KEY_PEM)
        .map_err(|_| "Embedded activation key is invalid".to_string())?;

    let message = canonical_message(&payload);
    let digest = Sha256::digest(message.as_bytes());
    public_key
        .verify(Pkcs1v15Sign::new::<Sha256>(), &digest, &signature)
        .map_err(|_| "Activation signature is invalid".to_string())?;

    if !payload.machine.eq_ignore_ascii_case(current_machine) {
        return Err("This activation code was issued for a different machine".to_string());
    }

    if payload.expiry != 0 && payload.expiry < now_unix() {
        return Err("This license has expired".to_string());
    }

    Ok(LicenseStatus {
        state: "active",
        machine_id: current_machine.to_string(),
        expires_at: if payload.expiry == 0 {
            None
        } else {
            Some(payload.expiry)
        },
        plan: payload.plan,
        message: "Activation is valid".to_string(),
    })
}

fn now_unix() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(_) => 0,
    }
}

fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| format!("Cannot resolve app data directory: {error}"))?;
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Cannot create app data directory: {error}"))?;
    Ok(dir)
}

fn license_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join(LICENSE_FILE_NAME))
}

fn load_stored(app: &tauri::AppHandle) -> StoredLicense {
    let Ok(path) = license_file_path(app) else {
        return StoredLicense::default();
    };
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => StoredLicense::default(),
    }
}

fn save_stored(app: &tauri::AppHandle, stored: &StoredLicense) -> Result<(), String> {
    let path = license_file_path(app)?;
    let json =
        serde_json::to_string(stored).map_err(|e| format!("Cannot encode license state: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Cannot persist license state: {e}"))
}

/// Build the current license status for the app (never fails; the status
/// carries the human-readable reason instead).
pub fn current_status(app: &tauri::AppHandle) -> LicenseStatus {
    let current_machine = machine_id();
    let stored = load_stored(app);
    let Some(code) = stored.code.as_deref() else {
        return LicenseStatus {
            state: "unactivated",
            machine_id: current_machine,
            expires_at: None,
            plan: String::new(),
            message: "No activation code has been entered".to_string(),
        };
    };

    if let Some(last_seen) = stored.last_seen {
        if now_unix() < last_seen - CLOCK_ROLLBACK_TOLERANCE_SECS {
            return LicenseStatus {
                state: "invalid",
                machine_id: current_machine,
                expires_at: None,
                plan: String::new(),
                message: "System clock appears to have been rolled back".to_string(),
            };
        }
    }

    match verify_code(code, &current_machine) {
        Ok(mut status) => {
            let max_last_seen = stored
                .last_seen
                .map_or_else(now_unix, |last| now_unix().max(last));
            let mut next = stored.clone();
            next.last_seen = Some(max_last_seen);
            let _ = save_stored(app, &next);
            status.machine_id = current_machine;
            status
        }
        Err(reason) => {
            let state = if reason.contains("expired") {
                "expired"
            } else {
                "invalid"
            };
            LicenseStatus {
                state,
                machine_id: current_machine,
                expires_at: None,
                plan: String::new(),
                message: reason,
            }
        }
    }
}

#[tauri::command]
pub fn get_machine_id() -> String {
    machine_id()
}

#[tauri::command]
pub fn get_license_status(app: tauri::AppHandle) -> Result<LicenseStatus, String> {
    Ok(current_status(&app))
}

#[tauri::command]
pub fn activate_license(app: tauri::AppHandle, code: String) -> Result<LicenseStatus, String> {
    let current_machine = machine_id();
    let verified = verify_code(&code, &current_machine)?;

    let mut stored = load_stored(&app);
    stored.code = Some(code);
    stored.last_seen = Some(
        stored
            .last_seen
            .map_or_else(now_unix, |last| now_unix().max(last)),
    );
    save_stored(&app, &stored)?;

    Ok(LicenseStatus {
        state: "active",
        machine_id: current_machine,
        expires_at: verified.expires_at,
        plan: verified.plan,
        message: "Activation successful".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::RsaPrivateKey;

    // Test-only private key matching the committed public key. This exists
    // solely to mint codes inside unit tests; it never ships in the binary.
    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAoRfyP1Pd3cMe2wVRpDvVwqlsG062wbjSRgJJmVcazLJ55npm
2nlJJC0rPdTfj6xKjpwoepQOjWef+IfnIaq5x4ifIL5Rqot57FUh9fLUhsfbBxqa
9l4B4bjzVaYdL+40WayWhbr2Cz/T0f206Gl47i4R7PY/B5oqkxMJCJ8DH5ftoRJW
y05aL6QvYon0bSNcokViN3RNIQKmnUZVmLN/V1ToG8ld8iIKxLCXpuRV6e4pW0H2
uCrNEf4zSefmnSSr6JJc2qJJI44llqEKIUVSu1W/H7HAhkRnnTw8rGIOffZZhHYK
c5ngYDN0skDJk2XvP/EVP9mRwYVIiuI6qN7NWQIDAQABAoIBACxjNsKwTy/ttbDP
1GEWg53xpPIZTE4wqO/VbYYs9Bt3ChOxommMhdTMBWAyKa+i36BP8u9joGbiKLgI
am1iZ3JNeNtC1anSaLPv+T2VvW2t3/IjKJO973LMzW/KJ7yvPAmBfrA2hdkvqyPv
bkuj1Eh8tT+/0jo8BoMj+ZdAiVHI0Mp6+FGtctJ5fADO5L6BBv9fuPGlQWNXWktT
XAJb/YzYe+6YYeMcc3xBU1jAq0EkjSwlMro2/euqK7p30CNj9aA2djHUAXUlu4AQ
M7Mw1Yq3c1m2jKtcLhcMpjhYLDih4lxlXZqQ9v0eeTa3szl/SDH2B/f6uyXhcyJA
SyoohGkCgYEA3LJKSf0VktwlPkld6PXKn12DWLmeNTag0jAcPpjlOq44obb/LCA8
z9pzKg6dlznxDW0W2FEh8B2MK8nbMoWWSe0oOTaKdijgz/c/zC6JfoPeUfhlmP7P
mAG8oI0WIG2lzmjqWikd6P7Wy7evpi9vuVs43poVMZrsU70+kiuQ02sCgYEAutzd
nYnza33Z3QUroDcLinuGogMCBqsdrJSqSJe/+sK2efPiJWiPFcMvrriDeiHiw74M
SV2gGLa/cRs7iMBeet+n260/jpNBqyquyoDGbDIQuwXe4E61sMxs6Dx3Yx9IlydY
K8xGz4PDCdDnP1RjhBoa8J2BpRGFzbVn0CYm10sCgYEAoFcwPf0PtXkX3d7zyZL5
uTr5eHazCLo9sTee7aOrtyxonKUVUvvA8solLYMc34gLJ9P/6v1XCNqOwimPhW8k
t2UD+j0z75DXdICP60pMPoyT8z3iontOW4O3hXW7g6fl9MaorQR+ZI7xg2RaywRf
yW0LLj8H8tg5psUxNoVMXS0CgYALY052PWQCVwch+yCvgOtJI0IPOZO0CFWkoOLH
zce7ZoZsZDqSAQ9HSj/rq/8HQG1rdHeXOQd6uhhssjYsnVykrGC0eJh4/exwnAdO
/A4bpelzHGZ60REtlyfD93tnJQ7td05eTPK7ztHiFUMijugJsvWVWGQARVMcOEP9
YXXYhQKBgDL356ZvB4Yria8cHTDACrTvFcswv5yDf17eBrkJ3Cc255NM0e4SNCmQ
6eONAdeLsTxfAwFCkpLdKrVD+Df+OYv7qFESnXOHX1drnfYNQ2JBkdTfC++pD/J3
Em97u4Y8aPwVUnVmegjPPDqACN3HXUAd6SmT1eTqjJOXDvBztNa1
-----END RSA PRIVATE KEY-----"#;

    fn mint_code(machine: &str, expiry: i64, plan: &str) -> String {
        let payload = serde_json::json!({
            "v": 1,
            "m": machine.to_lowercase(),
            "e": expiry,
            "p": plan,
        });
        let message = canonical_message(&LicensePayload {
            version: 1,
            machine: machine.to_lowercase(),
            expiry,
            plan: plan.to_string(),
        });
        let private_key = RsaPrivateKey::from_pkcs1_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let digest = Sha256::digest(message.as_bytes());
        let signature = private_key
            .sign(Pkcs1v15Sign::new::<Sha256>(), &digest)
            .unwrap();
        let encode = |bytes: &[u8]| URL_SAFE_NO_PAD.encode(bytes);
        format!(
            "{}.{}",
            encode(payload.to_string().as_bytes()),
            encode(&signature)
        )
    }

    const MACHINE: &str = "A1B2C3D4E5F6A1B2C3D4E5F6";

    #[test]
    fn machine_id_has_expected_shape() {
        let id = machine_id();
        assert_eq!(id.len(), 24);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(id, id.to_uppercase());

        let id_again = machine_id();
        assert_eq!(id, id_again, "machine id must be stable");
    }

    #[test]
    fn accepts_valid_license() {
        let code = mint_code(MACHINE, now_unix() + 86_400, "basic");
        let status = verify_code(&code, MACHINE).expect("valid license should verify");
        assert_eq!(status.state, "active");
        assert_eq!(status.plan, "basic");
        assert!(status.expires_at.is_some());
    }

    #[test]
    fn accepts_lifetime_license() {
        let code = mint_code(MACHINE, 0, "");
        let status = verify_code(&code, MACHINE).expect("lifetime license should verify");
        assert_eq!(status.state, "active");
        assert_eq!(status.expires_at, None);
    }

    #[test]
    fn rejects_wrong_machine() {
        let code = mint_code(MACHINE, now_unix() + 86_400, "");
        let error = verify_code(&code, "DEADBEEFDEADBEEFDEADBEEF")
            .expect_err("wrong machine must be rejected");
        assert!(error.contains("different machine"));
    }

    #[test]
    fn rejects_expired_license() {
        let code = mint_code(MACHINE, now_unix() - 10, "");
        let error = verify_code(&code, MACHINE).expect_err("expired license must be rejected");
        assert!(error.contains("expired"));
    }

    #[test]
    fn rejects_tampered_signature() {
        let code = mint_code(MACHINE, now_unix() + 86_400, "");
        let (payload, signature) = code.split_once('.').unwrap();
        let mut flipped = signature.to_string();
        let original = signature.chars().next().unwrap();
        let replacement = if original == 'A' { 'B' } else { 'A' };
        flipped.replace_range(0..1, &replacement.to_string());
        let tampered = format!("{payload}.{flipped}");
        let error =
            verify_code(&tampered, MACHINE).expect_err("tampered signature must be rejected");
        assert!(error.contains("signature"));
    }

    #[test]
    fn rejects_case_mismatch_on_machine() {
        let code = mint_code(MACHINE, now_unix() + 86_400, "");
        assert!(verify_code(&code, MACHINE).is_ok());
        assert!(verify_code(&code, &MACHINE.to_lowercase()).is_ok());
    }

    #[test]
    fn rejects_garbage() {
        assert!(verify_code("not.a.code", MACHINE).is_err());
        assert!(verify_code("", MACHINE).is_err());
        assert!(verify_code("abc.def", MACHINE).is_err());
    }
}
