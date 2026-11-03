import { invoke } from '@tauri-apps/api/core'

/** One badge Discord reports on the account's profile. */
export interface ProfileBadge {
  id: string
  description: string
  /** Icon hash, resolved through Discord's badge-icon CDN. */
  icon: string
  link: string | null
}

/** Badge overview computed by the Rust backend from Discord data and local history. */
export interface BadgeOverview {
  userId: string
  /** `public_flags` bitfield from the Discord user object. */
  flags: number
  /** Nitro subscription type: 0 = none, 1 = Classic, 2 = Nitro, 3 = Basic. */
  premiumType: number
  /** Account creation time in Unix milliseconds (decoded from the snowflake). */
  accountCreatedMs: number
  accountAgeDays: number
  accountAgeYears: number
  /** 0 = under one year, 10 = ten years or more. */
  accountAgeLevel: number
  /** Progress towards the next tier, 0..1. */
  accountAgeProgress: number
  /** Years needed for the next tier, `null` once maxed. */
  accountAgeNextYears: number | null
  /** Badges Discord reports on the profile, exactly as returned. */
  profileBadges: ProfileBadge[]
  /** `false` when the profile endpoint was unavailable and flags are used instead. */
  profileBadgesAvailable: boolean
  /** Playtime recorded by this app for the account, in seconds. */
  trackedGameSeconds: number
  /** Distinct games with recorded playtime in this app. */
  trackedGameCount: number
  gameTimeHours: number
  /** 0 = under one hour, 10 = 5,000 hours or more. */
  gameTimeLevel: number
  gameTimeProgress: number
  gameTimeNextHours: number | null
  /** 0 = fewer than two games, 10 = 100 games or more. */
  gameVarietyLevel: number
  gameVarietyProgress: number
  gameVarietyNextGames: number | null
  earnedBadgeIds: string[]
}

export function getBadgeOverview(): Promise<BadgeOverview> {
  return invoke('get_badge_overview')
}
