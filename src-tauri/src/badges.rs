//! Profile badge overview.
//!
//! Three data sources are merged here, and each one is labelled for what it is:
//!
//! * Discord's read-only profile endpoint, which reports the badges that are
//!   actually on the account (`profile_badges`).
//! * The public user payload (`public_flags`, Nitro type), used as a fallback
//!   when the profile call is unavailable.
//! * The account creation timestamp encoded in the user snowflake, which gives
//!   the account-age tier.
//!
//! Leveled badges such as Game Time and Game Variety are computed privately by
//! Discord and are not readable through any API. Their progress here is derived
//! from the play sessions this app recorded locally, so it is reported as
//! app-tracked data rather than as Discord's own counters.

use crate::models::{DiscordUser, ProfileBadge};
use crate::simulation_history::GameSimulationHistoryEntry;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Discord epoch (2015-01-01T00:00:00Z) in milliseconds.
const DISCORD_EPOCH_MS: i64 = 1_420_070_400_000;
const MS_PER_DAY: f64 = 86_400_000.0;
const DAYS_PER_YEAR: f64 = 365.2425;
const SECONDS_PER_HOUR: f64 = 3_600.0;
/// Account age badges stop at ten years.
const MAX_ACCOUNT_AGE_LEVEL: u8 = 10;

/// Minimum hours for each Game Time tier, as Discord lists them.
pub const GAME_TIME_MIN_HOURS: [u64; 10] = [1, 5, 20, 75, 150, 300, 500, 1_000, 2_000, 5_000];
/// Minimum distinct games for each Game Variety tier.
pub const GAME_VARIETY_MIN_GAMES: [u64; 10] = [2, 5, 10, 15, 20, 30, 40, 60, 80, 100];

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
    /// Badges Discord reports on the profile, exactly as returned.
    pub profile_badges: Vec<ProfileBadge>,
    /// `false` when the profile endpoint could not be reached and the badge
    /// list falls back to `earned_badge_ids`.
    pub profile_badges_available: bool,
    /// Total playtime recorded by this app for the account, in seconds.
    pub tracked_game_seconds: u64,
    /// Distinct games with recorded playtime in this app.
    pub tracked_game_count: u32,
    pub game_time_hours: f64,
    pub game_time_level: u8,
    pub game_time_progress: f64,
    pub game_time_next_hours: Option<u64>,
    pub game_variety_level: u8,
    pub game_variety_progress: f64,
    pub game_variety_next_games: Option<u64>,
    /// Flag/Nitro-derived badges. Used when `profile_badges_available` is false.
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

/// Current tier, progress towards the next tier, and the next threshold.
///
/// A value below the first threshold maps to level 0. Once every threshold is
/// reached the progress is 1.0 and there is no next threshold.
pub fn leveled_progress(value: f64, thresholds: &[u64]) -> (u8, f64, Option<u64>) {
    let value = value.max(0.0);
    let mut level = 0usize;
    for threshold in thresholds {
        if value >= *threshold as f64 {
            level += 1;
        } else {
            break;
        }
    }

    if level >= thresholds.len() {
        return (thresholds.len() as u8, 1.0, None);
    }

    let floor = if level == 0 {
        0.0
    } else {
        thresholds[level - 1] as f64
    };
    let ceiling = thresholds[level] as f64;
    let progress = ((value - floor) / (ceiling - floor)).clamp(0.0, 1.0);
    (level as u8, progress, Some(thresholds[level]))
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

/// Build the badge overview from the user payload, the profile badge list and
/// the locally recorded play history.
pub fn overview(
    user: &DiscordUser,
    profile_badges: Vec<ProfileBadge>,
    profile_badges_available: bool,
    history: &[GameSimulationHistoryEntry],
) -> BadgeOverview {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0);

    let created_ms = account_created_ms(&user.id).unwrap_or(now_ms);
    let age_days = days_between(now_ms, created_ms);
    let level = account_age_level(age_days);
    let premium_type = user.premium_type.unwrap_or(0);
    let flags = user.public_flags.unwrap_or(0);

    let tracked_game_seconds: u64 = history.iter().map(|entry| entry.total_seconds).sum();
    let tracked_game_count = history
        .iter()
        .filter(|entry| entry.total_seconds > 0)
        .count() as u32;
    let game_time_hours = tracked_game_seconds as f64 / SECONDS_PER_HOUR;
    let (game_time_level, game_time_progress, game_time_next_hours) =
        leveled_progress(game_time_hours, &GAME_TIME_MIN_HOURS);
    let (game_variety_level, game_variety_progress, game_variety_next_games) =
        leveled_progress(tracked_game_count as f64, &GAME_VARIETY_MIN_GAMES);

    BadgeOverview {
        user_id: user.id.clone(),
        flags,
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
        profile_badges,
        profile_badges_available,
        tracked_game_seconds,
        tracked_game_count,
        game_time_hours,
        game_time_level,
        game_time_progress,
        game_time_next_hours,
        game_variety_level,
        game_variety_progress,
        game_variety_next_games,
        earned_badge_ids: earned_badge_ids(flags, premium_type),
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

    fn history_entry(app_id: &str, seconds: u64) -> GameSimulationHistoryEntry {
        GameSimulationHistoryEntry {
            app_id: app_id.to_string(),
            app_name: format!("Game {}", app_id),
            total_seconds: seconds,
            updated_at: "2026-01-01T00:00:00Z".to_string(),
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
    fn leveled_progress_matches_thresholds() {
        assert_eq!(
            leveled_progress(0.0, &GAME_TIME_MIN_HOURS),
            (0, 0.0, Some(1))
        );
        assert_eq!(leveled_progress(1.0, &GAME_TIME_MIN_HOURS).0, 1);
        assert_eq!(leveled_progress(4.5, &GAME_TIME_MIN_HOURS).0, 1);
        assert_eq!(leveled_progress(5.0, &GAME_TIME_MIN_HOURS).0, 2);
        assert_eq!(
            leveled_progress(5_000.0, &GAME_TIME_MIN_HOURS),
            (10, 1.0, None)
        );
    }

    #[test]
    fn leveled_progress_reports_next_threshold() {
        let (level, progress, next) = leveled_progress(2.5, &GAME_TIME_MIN_HOURS);
        assert_eq!(level, 1);
        assert_eq!(next, Some(5));
        assert!((progress - 0.375).abs() < f64::EPSILON);
    }

    #[test]
    fn overview_uses_user_flags_and_age() {
        let created = 1_600_000_000_000;
        let overview = overview(
            &user(&snowflake_for(created), FLAG_PARTNER, 0),
            Vec::new(),
            false,
            &[],
        );
        assert_eq!(overview.flags, FLAG_PARTNER);
        assert_eq!(overview.account_created_ms, created);
        assert!(overview.earned_badge_ids.contains(&"partner".to_string()));
        assert!(overview.account_age_next_years.is_some());
        assert!(!overview.profile_badges_available);
    }

    #[test]
    fn overview_aggregates_tracked_playtime() {
        let created = 1_600_000_000_000;
        let history = vec![
            history_entry("1", 3_600),
            history_entry("2", 3_600),
            history_entry("3", 0),
        ];
        let overview = overview(
            &user(&snowflake_for(created), 0, 0),
            Vec::new(),
            true,
            &history,
        );
        assert_eq!(overview.tracked_game_seconds, 7_200);
        assert_eq!(overview.tracked_game_count, 2);
        assert_eq!(overview.game_time_hours, 2.0);
        assert_eq!(overview.game_time_level, 1);
        assert_eq!(overview.game_variety_level, 1);
        assert_eq!(overview.game_variety_next_games, Some(5));
    }

    #[test]
    fn overview_passes_profile_badges_through() {
        let created = 1_600_000_000_000;
        let badge = ProfileBadge {
            id: "quest_completed".to_string(),
            description: "Completed a Quest".to_string(),
            icon: "abc".to_string(),
            link: None,
        };
        let overview = overview(
            &user(&snowflake_for(created), 0, 0),
            vec![badge.clone()],
            true,
            &[],
        );
        assert!(overview.profile_badges_available);
        assert_eq!(overview.profile_badges, vec![badge]);
    }
}
