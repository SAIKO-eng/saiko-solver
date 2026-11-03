import { describe, expect, it } from 'vitest'
import {
  ACCOUNT_AGE_TIERS,
  GAME_TIME_TIERS,
  GAME_VARIETY_TIERS,
  GIFTING_TIERS,
  STREAMING_TIERS,
  accountAgeTier,
  badgeIconUrl,
  leveledTier,
  nextLeveledTier,
  type LeveledTier,
} from './badgeCatalog'

function expectAscending(tiers: LeveledTier[]) {
  expect(tiers.length).toBeGreaterThan(0)
  for (let index = 1; index < tiers.length; index += 1) {
    expect(tiers[index].level).toBe(tiers[index - 1].level + 1)
    expect(tiers[index].threshold).toBeGreaterThan(tiers[index - 1].threshold)
  }
}

describe('badge catalog', () => {
  it('lists every leveled track in ascending order', () => {
    expect(GAME_TIME_TIERS).toHaveLength(10)
    expect(GAME_VARIETY_TIERS).toHaveLength(10)
    expect(STREAMING_TIERS).toHaveLength(10)
    expect(GIFTING_TIERS).toHaveLength(6)
    for (const tiers of [GAME_TIME_TIERS, GAME_VARIETY_TIERS, STREAMING_TIERS, GIFTING_TIERS]) {
      expectAscending(tiers)
    }
  })

  it('matches Discord account-age tiers', () => {
    expect(ACCOUNT_AGE_TIERS).toHaveLength(10)
    expect(accountAgeTier(0)).toBeNull()
    expect(accountAgeTier(1)?.name).toBe('Seed')
    expect(accountAgeTier(10)?.name).toBe('Primordial')
    expect(accountAgeTier(11)).toBeNull()
  })

  it('resolves current and next tiers', () => {
    expect(leveledTier(GAME_TIME_TIERS, 0)).toBeNull()
    expect(leveledTier(GAME_TIME_TIERS, 1)?.name).toBe('Casual')
    expect(leveledTier(GAME_TIME_TIERS, 10)?.name).toBe('Eternal')
    expect(nextLeveledTier(GAME_TIME_TIERS, 0)?.name).toBe('Casual')
    expect(nextLeveledTier(GAME_TIME_TIERS, 10)).toBeNull()
  })

  it('builds Discord badge icon URLs only for real hashes', () => {
    expect(badgeIconUrl('')).toBeNull()
    expect(badgeIconUrl('   ')).toBeNull()
    expect(badgeIconUrl('5e74e9b61934fc1f67c65515d1f7e60d')).toBe(
      'https://cdn.discordapp.com/badge-icons/5e74e9b61934fc1f67c65515d1f7e60d.png',
    )
  })
})
