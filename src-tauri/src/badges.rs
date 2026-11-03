//! Profile badge overview.
//!
//! Discord only exposes classic account badges through the public API
//! (`public_flags` on the user object). Leveled badges such as Game Time and
//! Game Variety are computed server-side and are not part of the public
//! payload, so this module reports only what can be derived truthfully:
//! the flag-based badges, the Nitro badge, and the account-age tier which is
//! derived from the creation timestamp encoded in the user snowflake.

use crate::models::DiscordUser;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Discord epoch (2015-01-01T00:00:00Z) in milliseconds.
const DISCORD_EPOCH_MS: i64 = 1_420_070_400_000;
const MS_PER_DAY: f64 = 86_400_000.0;
const DAYS_PER_YEAR: f64 = 365.2425;
/// Account age badges stop at ten years.
const MAX_ACCOUNT_AGE_LEVEL: u8 = 10;

pub const FLAG_STAFF: u64 = 1 << 0;
pub const FLAG_PARTNER: u64 = 1 << 1;
pub const FLAG_HYPESQUAD_EVENTS: u64 = 1 << 2;
pub const FLAG_BUG_HUNTER_LEVEL_1: u64 = 1 << 3;
pub const FLAG_HYPESQUAD_BRAVERY: u64 = 1 << 6;
pub const FLAG_HYPESQUAD_BRILLIANCE: u64 = 1 << 7;
pub const FLAG_HYPESQUAD_BALANCE: u64 = 1 << 8;
pub const FLAG_EARLY_SUPPORTER: u64 = 1 << 9;
pub const FLAG_BUG_HUNTER_LEVEL_2: u64 = 1 << 14;
pub const FLAG_VERIFIED_DEVELOPER: u64 = 1 << 17;
pub const FLAG_CERTIFIED_MODERATOR: u64 = 1 << 18;
pub const FLAG_ACTIVE_DEVELOPER: u64 = 1 << 22;

/// Everything the badges view needs to render one account.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeOverview {
    pub user_id: String,
    pub flags: u64,
    pub premium_type: u8,
    /// Account creation time in Unix milliseconds.
    pub account_created_ms: i64,
    pub account_age_days: i64,
    pub account_age_years: f64,
    /// Current account-age tier (0 = under one year, 10 = ten years or more).
    pub account_age_level: u8,
    /// Progress towards the next tier, in the range 0..=1.
    pub account_age_progress: f64,
    /// Years required for the next tier, `None` once the badge is maxed.
    pub account_age_next_years: Option<u8>,
    pub earned_badge_ids: Vec<String>,
}

/// Decode the account creation timestamp encoded in a Discord snowflake.
pub fn account_created_ms(user_id: &str) -> Option<i64> {
    let id: u64 = user_id.trim().parse().ok()?;
    if id == 0 {
        return None;
    }
    Some(((id >> 22) as i64) + DISCORD_EPOCH_MS)
}

/// Whole days between two Unix millisecond timestamps, never negative.
pub fn days_between(now_ms: i64, created_ms: i64) -> i64 {
    (now_ms - created_ms).max(0) / MS_PER_DAY as i64
}

/// Account-age tier for a given number of days.
pub fn account_age_level(days: i64) -> u8 {
    let years = (days as f64 / DAYS_PER_YEAR).floor();
    years.clamp(0.0, MAX_ACCOUNT_AGE_LEVEL as f64) as u8
}

/// Progress towards the next account-age tier.
pub fn account_age_progress(days: i64) -> f64 {
    let years = days as f64 / DAYS_PER_YEAR;
    if years >= MAX_ACCOUNT_AGE_LEVEL as f64 {
        return 1.0;
    }
    (years - years.floor()).clamp(0.0, 1.0)
}

/// Badge ids earned through `public_flags` and the Nitro subscription.
pub fn earned_badge_ids(flags: u64, premium_type: u8) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();

    if flags & FLAG_STAFF != 0 {
        ids.push("staff".to_string());
    }
    if flags & FLAG_PARTNER != 0 {
        ids.push("partner".to_string());
    }
    if flags & FLAG_HYPESQUAD_EVENTS != 0 {
        ids.push("hypesquad_events".to_string());
    }
    if flags & FLAG_HYPESQUAD_BRAVERY != 0 {
        ids.push("hypesquad_bravery".to_string());
    }
    if flags & FLAG_HYPESQUAD_BRILLIANCE != 0 {
        ids.push("hypesquad_brilliance".to_string());
    }
    if flags & FLAG_HYPESQUAD_BALANCE != 0 {
        ids.push("hypesquad_balance".to_string());
    }
    if flags & FLAG_BUG_HUNTER_LEVEL_1 != 0 {
        ids.push("bug_hunter_1".to_string());
    }
    if flags & FLAG_BUG_HUNTER_LEVEL_2 != 0 {
        ids.push("bug_hunter_2".to_string());
    }
    if flags & FLAG_EARLY_SUPPORTER != 0 {
        ids.push("early_supporter".to_string());
    }
    if flags & FLAG_VERIFIED_DEVELOPER != 0 {
        ids.push("verified_developer".to_string());
    }
    if flags & FLAG_CERTIFIED_MODERATOR != 0 {
        ids.push("certified_moderator".to_string());
    }
    if flags & FLAG_ACTIVE_DEVELOPER != 0 {
        ids.push("active_developer".to_string());
    }

    match premium_type {
        1 => ids.push("nitro_classic".to_string()),
        2 => ids.push("nitro".to_string()),
        3 => ids.push("nitro_basic".to_string()),
        _ => {}
    }

    ids
}

/// Build the badge overview for a user payload.
pub fn overview(user: &DiscordUser) -> BadgeOverview {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0);

    let created_ms = account_created_ms(&user.id).unwrap_or(now_ms);
    let age_days = days_between(now_ms, created_ms);
    let level = account_age_level(age_days);
    let premium_type = user.premium_type.unwrap_or(0);

    BadgeOverview {
        user_id: user.id.clone(),
        flags: user.public_flags.unwrap_or(0),
        premium_type,
        account_created_ms: created_ms,
        account_age_days: age_days,
        account_age_years: age_days as f64 / DAYS_PER_YEAR,
        account_age_level: level,
        account_age_progress: account_age_progress(age_days),
        account_age_next_years: if level >= MAX_ACCOUNT_AGE_LEVEL {
            None
        } else {
            Some(level + 1)
        },
        earned_badge_ids: earned_badge_ids(user.public_flags.unwrap_or(0), premium_type),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snowflake_for(ms: i64) -> String {
        (((ms - DISCORD_EPOCH_MS) as u64) << 22 | 1).to_string()
    }

    fn user(id: &str, flags: u64, premium_type: u8) -> DiscordUser {
        DiscordUser {
            id: id.to_string(),
            username: "tester".to_string(),
            discriminator: "0".to_string(),
            avatar: None,
            global_name: None,
            premium_type: Some(premium_type),
            public_flags: Some(flags),
        }
    }

    #[test]
    fn decodes_snowflake_timestamp() {
        let created = 1_600_000_000_000;
        let id = snowflake_for(created);
        assert_eq!(account_created_ms(&id), Some(created));
    }

    #[test]
    fn rejects_invalid_snowflakes() {
        assert_eq!(account_created_ms("not-a-number"), None);
        assert_eq!(account_created_ms("0"), None);
    }

    #[test]
    fn account_age_levels_follow_year_boundaries() {
        assert_eq!(account_age_level(0), 0);
        assert_eq!(account_age_level(364), 0);
        assert_eq!(account_age_level(366), 1);
        assert_eq!(account_age_level(365 * 5 + 3), 5);
        assert_eq!(account_age_level(365 * 30), 10);
    }

    #[test]
    fn account_age_progress_is_bounded() {
        assert!((0.0..=1.0).contains(&account_age_progress(0)));
        assert!((0.0..=1.0).contains(&account_age_progress(400)));
        assert_eq!(account_age_progress(365 * 12), 1.0);
    }

    #[test]
    fn maps_flags_and_nitro_to_badge_ids() {
        let ids = earned_badge_ids(
            FLAG_STAFF | FLAG_BUG_HUNTER_LEVEL_1 | FLAG_ACTIVE_DEVELOPER,
            2,
        );
        assert!(ids.contains(&"staff".to_string()));
        assert!(ids.contains(&"bug_hunter_1".to_string()));
        assert!(ids.contains(&"active_developer".to_string()));
        assert!(ids.contains(&"nitro".to_string()));
        assert_eq!(ids.len(), 4);
    }

    #[test]
    fn overview_uses_user_flags_and_age() {
        let created = 1_600_000_000_000;
        let overview = overview(&user(&snowflake_for(created), FLAG_PARTNER, 0));
        assert_eq!(overview.flags, FLAG_PARTNER);
        assert_eq!(overview.account_created_ms, created);
        assert!(overview.earned_badge_ids.contains(&"partner".to_string()));
        assert!(overview.account_age_next_years.is_some());
    }
}
