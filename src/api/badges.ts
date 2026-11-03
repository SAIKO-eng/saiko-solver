import { invoke } from '@tauri-apps/api/core'

/** Badge overview computed by the Rust backend from the Discord user payload. */
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
  earnedBadgeIds: string[]
}

export function getBadgeOverview(): Promise<BadgeOverview> {
  return invoke('get_badge_overview')
}
