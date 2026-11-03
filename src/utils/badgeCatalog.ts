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

export function accountAgeTier(level: number): AccountAgeTier | null {
  if (level <= 0) return null
  return ACCOUNT_AGE_TIERS.find(tier => tier.level === level) ?? null
}
