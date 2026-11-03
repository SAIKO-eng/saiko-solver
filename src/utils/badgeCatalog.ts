/**
 * Static catalog for the badges view.
 *
 * Badge artwork lives in `public/badges/` and comes from the MIT-licensed
 * `mezotv/discord-badges` collection. Names are Discord's own badge names, so
 * they stay in English in every locale.
 */

export interface BadgeDefinition {
  /** Id returned by the backend in `earnedBadgeIds`. */
  id: string
  name: string
  asset: string
}

export interface AccountAgeTier {
  level: number
  name: string
  years: number
  asset: string
}

export interface LeveledTier {
  level: number
  name: string
  /** Hours of playtime (Game Time, Streaming) or number of games (Game Variety). */
  threshold: number
  asset: string
}

export const BADGE_CATALOG: BadgeDefinition[] = [
  { id: 'staff', name: 'Discord Staff', asset: '/badges/discord-staff.svg' },
  { id: 'partner', name: 'Partnered Server Owner', asset: '/badges/discord-partner.svg' },
  { id: 'hypesquad_events', name: 'HypeSquad Events', asset: '/badges/hype-squad-events.svg' },
  { id: 'hypesquad_bravery', name: 'HypeSquad Bravery', asset: '/badges/hype-squad-bravery.svg' },
  {
    id: 'hypesquad_brilliance',
    name: 'HypeSquad Brilliance',
    asset: '/badges/hype-squad-brilliance.svg',
  },
  { id: 'hypesquad_balance', name: 'HypeSquad Balance', asset: '/badges/hype-squad-balance.svg' },
  { id: 'bug_hunter_1', name: 'Bug Hunter', asset: '/badges/discord-bug-hunter-green.svg' },
  { id: 'bug_hunter_2', name: 'Bug Hunter Gold', asset: '/badges/discord-bug-hunter-gold.svg' },
  { id: 'early_supporter', name: 'Early Supporter', asset: '/badges/discord-early-supporter.svg' },
  {
    id: 'verified_developer',
    name: 'Early Verified Bot Developer',
    asset: '/badges/discord-bot-dev.svg',
  },
  {
    id: 'certified_moderator',
    name: 'Moderator Programs Alumni',
    asset: '/badges/discord-mod.svg',
  },
  { id: 'active_developer', name: 'Active Developer', asset: '/badges/active-developer.svg' },
  { id: 'nitro', name: 'Discord Nitro', asset: '/badges/discord-nitro.svg' },
  { id: 'nitro_classic', name: 'Discord Nitro Classic', asset: '/badges/discord-nitro.svg' },
  { id: 'nitro_basic', name: 'Discord Nitro Basic', asset: '/badges/discord-nitro-basic.svg' },
]

export const ACCOUNT_AGE_TIERS: AccountAgeTier[] = [
  { level: 1, name: 'Seed', years: 1, asset: '/badges/account-age-seed.svg' },
  { level: 2, name: 'Sprout', years: 2, asset: '/badges/account-age-sprout.svg' },
  { level: 3, name: 'Bud', years: 3, asset: '/badges/account-age-bud.svg' },
  { level: 4, name: 'Sapling', years: 4, asset: '/badges/account-age-sapling.svg' },
  { level: 5, name: 'Blossom', years: 5, asset: '/badges/account-age-blossom.svg' },
  { level: 6, name: 'Redwood', years: 6, asset: '/badges/account-age-redwood.svg' },
  { level: 7, name: 'Sequoia', years: 7, asset: '/badges/account-age-sequoia.svg' },
  { level: 8, name: 'Bristlecone', years: 8, asset: '/badges/account-age-bristlecone.svg' },
  { level: 9, name: 'Stromatolite', years: 9, asset: '/badges/account-age-stromatolite.svg' },
  { level: 10, name: 'Primordial', years: 10, asset: '/badges/account-age-primordial.svg' },
]

/** Thresholds match Discord's Game Time badge timeline. */
export const GAME_TIME_TIERS: LeveledTier[] = [
  { level: 1, name: 'Casual', threshold: 1, asset: '/badges/game-time-casual.svg' },
  { level: 2, name: 'Recreational', threshold: 5, asset: '/badges/game-time-recreational.svg' },
  { level: 3, name: 'Dedicated', threshold: 20, asset: '/badges/game-time-dedicated.svg' },
  { level: 4, name: 'Committed', threshold: 75, asset: '/badges/game-time-committed.svg' },
  { level: 5, name: 'Serious', threshold: 150, asset: '/badges/game-time-serious.svg' },
  { level: 6, name: 'Devoted', threshold: 300, asset: '/badges/game-time-devoted.svg' },
  { level: 7, name: 'Seasoned', threshold: 500, asset: '/badges/game-time-seasoned.svg' },
  { level: 8, name: 'Ironclad', threshold: 1000, asset: '/badges/game-time-ironclad.svg' },
  { level: 9, name: 'Unshakeable', threshold: 2000, asset: '/badges/game-time-unshakeable.svg' },
  { level: 10, name: 'Eternal', threshold: 5000, asset: '/badges/game-time-eternal.svg' },
]

/** Thresholds match Discord's Game Variety badge timeline. */
export const GAME_VARIETY_TIERS: LeveledTier[] = [
  { level: 1, name: 'Sampler', threshold: 2, asset: '/badges/game-variety-sampler.svg' },
  { level: 2, name: 'Dabbler', threshold: 5, asset: '/badges/game-variety-dabbler.svg' },
  { level: 3, name: 'Enthusiast', threshold: 10, asset: '/badges/game-variety-enthusiast.svg' },
  { level: 4, name: 'Ranger', threshold: 15, asset: '/badges/game-variety-ranger.svg' },
  { level: 5, name: 'Explorer', threshold: 20, asset: '/badges/game-variety-explorer.svg' },
  { level: 6, name: 'Adventurer', threshold: 30, asset: '/badges/game-variety-adventurer.svg' },
  { level: 7, name: 'Voyager', threshold: 40, asset: '/badges/game-variety-voyager.svg' },
  { level: 8, name: 'Maverick', threshold: 60, asset: '/badges/game-variety-maverick.svg' },
  { level: 9, name: 'Polymath', threshold: 80, asset: '/badges/game-variety-polymath.svg' },
  {
    level: 10,
    name: 'Universalist',
    threshold: 100,
    asset: '/badges/game-variety-universalist.svg',
  },
]

/** Streaming tiers. Progress is not exposed by Discord, so these are informational. */
export const STREAMING_TIERS: LeveledTier[] = [
  { level: 1, name: 'Newcomer', threshold: 1, asset: '/badges/streaming-newcomer.svg' },
  { level: 2, name: 'Fledgling', threshold: 5, asset: '/badges/streaming-fledgling.svg' },
  { level: 3, name: 'Breakout', threshold: 20, asset: '/badges/streaming-breakout.svg' },
  { level: 4, name: 'Standout', threshold: 75, asset: '/badges/streaming-standout.svg' },
  { level: 5, name: 'Trendsetter', threshold: 150, asset: '/badges/streaming-trendsetter.svg' },
  { level: 6, name: 'Headliner', threshold: 300, asset: '/badges/streaming-headliner.svg' },
  { level: 7, name: 'Star', threshold: 500, asset: '/badges/streaming-star.svg' },
  { level: 8, name: 'Sensation', threshold: 1000, asset: '/badges/streaming-sensation.svg' },
  { level: 9, name: 'Visionary', threshold: 2000, asset: '/badges/streaming-visionary.svg' },
  { level: 10, name: 'Phenomenon', threshold: 5000, asset: '/badges/streaming-phenomenon.svg' },
]

/** Gifting tiers. Counts are private, so these are informational. */
export const GIFTING_TIERS: LeveledTier[] = [
  { level: 1, name: 'Patron', threshold: 1, asset: '/badges/gifting-patron.png' },
  { level: 2, name: 'Champion', threshold: 2, asset: '/badges/gifting-champion.png' },
  { level: 3, name: 'Luminary', threshold: 3, asset: '/badges/gifting-luminary.png' },
  { level: 4, name: 'Icon', threshold: 6, asset: '/badges/gifting-icon.png' },
  { level: 5, name: 'Hero', threshold: 10, asset: '/badges/gifting-hero.png' },
  { level: 6, name: 'Legend', threshold: 20, asset: '/badges/gifting-legend.png' },
]

export function accountAgeTier(level: number): AccountAgeTier | null {
  if (level <= 0) return null
  return ACCOUNT_AGE_TIERS.find(tier => tier.level === level) ?? null
}

export function leveledTier(tiers: LeveledTier[], level: number): LeveledTier | null {
  if (level <= 0) return null
  return tiers.find(tier => tier.level === level) ?? null
}

export function nextLeveledTier(tiers: LeveledTier[], level: number): LeveledTier | null {
  return tiers.find(tier => tier.level === level + 1) ?? null
}

/** Resolve a profile badge icon hash to Discord's CDN URL. */
export function badgeIconUrl(icon: string): string | null {
  const hash = icon.trim()
  if (!hash) return null
  return `https://cdn.discordapp.com/badge-icons/${hash}.png`
}
