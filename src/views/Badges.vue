<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RefreshCw } from 'lucide-vue-next'
import { getBadgeOverview, type BadgeOverview, type ProfileBadge } from '@/api/badges'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import {
  ACCOUNT_AGE_TIERS,
  BADGE_CATALOG,
  GAME_TIME_TIERS,
  GAME_VARIETY_TIERS,
  GIFTING_TIERS,
  STREAMING_TIERS,
  accountAgeTier,
  badgeIconUrl,
  leveledTier,
  nextLeveledTier,
  type LeveledTier,
} from '@/utils/badgeCatalog'

const { t, locale } = useI18n()

const overview = ref<BadgeOverview | null>(null)
const loading = ref(true)
const error = ref('')

const earned = computed(() => new Set(overview.value?.earnedBadgeIds ?? []))
const currentTier = computed(() => accountAgeTier(overview.value?.accountAgeLevel ?? 0))
const nextTier = computed(() => {
  const years = overview.value?.accountAgeNextYears
  if (!years) return null
  return ACCOUNT_AGE_TIERS.find(tier => tier.years === years) ?? null
})
const accountAgePercent = computed(() =>
  Math.round((overview.value?.accountAgeProgress ?? 0) * 100),
)

const gameTimeTier = computed(() =>
  leveledTier(GAME_TIME_TIERS, overview.value?.gameTimeLevel ?? 0),
)
const gameTimeNext = computed(() =>
  nextLeveledTier(GAME_TIME_TIERS, overview.value?.gameTimeLevel ?? 0),
)
const gameTimePercent = computed(() => Math.round((overview.value?.gameTimeProgress ?? 0) * 100))
const gameTimeHours = computed(() => overview.value?.gameTimeHours ?? 0)

const gameVarietyTier = computed(() =>
  leveledTier(GAME_VARIETY_TIERS, overview.value?.gameVarietyLevel ?? 0),
)
const gameVarietyNext = computed(() =>
  nextLeveledTier(GAME_VARIETY_TIERS, overview.value?.gameVarietyLevel ?? 0),
)
const gameVarietyPercent = computed(() =>
  Math.round((overview.value?.gameVarietyProgress ?? 0) * 100),
)
const gameVarietyCount = computed(() => overview.value?.trackedGameCount ?? 0)

const createdAt = computed(() => {
  const ms = overview.value?.accountCreatedMs
  if (!ms) return ''
  try {
    return new Intl.DateTimeFormat(locale.value, { dateStyle: 'medium' }).format(new Date(ms))
  } catch {
    return new Date(ms).toLocaleDateString()
  }
})
const ageYears = computed(() => Math.floor(overview.value?.accountAgeYears ?? 0))

const profileBadges = computed(() => overview.value?.profileBadges ?? [])

const numberFormat = computed(() => new Intl.NumberFormat(locale.value))
function formatNumber(value: number): string {
  try {
    return numberFormat.value.format(value)
  } catch {
    return String(value)
  }
}
function hoursLabel(value: number): string {
  return t('badges.hours', { value: formatNumber(value) })
}
function gamesLabel(value: number): string {
  return t('badges.games', { count: formatNumber(value) })
}
function nextLabel(tier: LeveledTier, unit: 'hours' | 'games'): string {
  return t('badges.next', {
    name: tier.name,
    target: unit === 'hours' ? hoursLabel(tier.threshold) : gamesLabel(tier.threshold),
  })
}
function badgeIcon(badge: ProfileBadge): string | null {
  return badgeIconUrl(badge.icon)
}
function initial(badge: ProfileBadge): string {
  const source = badge.description || badge.id
  return source.slice(0, 1).toUpperCase()
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    overview.value = await getBadgeOverview()
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err)
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="badges-view fade-in space-y-6 select-none">
    <div class="flex min-w-0 items-start justify-between gap-4">
      <div class="min-w-0 space-y-2">
        <h2 class="text-2xl font-bold tracking-tight">{{ t('badges.title') }}</h2>
        <p class="text-sm text-muted-foreground text-pretty">{{ t('badges.subtitle') }}</p>
      </div>
      <Button variant="outline" size="sm" :disabled="loading" @click="load">
        <RefreshCw class="mr-2 h-4 w-4" :class="{ 'animate-spin': loading }" />
        {{ t('badges.refresh') }}
      </Button>
    </div>

    <div
      v-if="error"
      class="rounded-lg border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive"
    >
      <p>{{ t('badges.error') }}</p>
      <p class="mt-1 break-words text-xs opacity-80">{{ error }}</p>
    </div>

    <div v-else-if="loading && !overview" class="text-sm text-muted-foreground">
      {{ t('badges.loading') }}
    </div>

    <template v-else-if="overview">
      <!-- Badges reported by Discord -->
      <section class="space-y-3">
        <div class="flex items-baseline justify-between gap-3">
          <h3 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
            {{ t('badges.profile_badges') }}
          </h3>
          <span v-if="overview.profileBadgesAvailable" class="text-xs text-muted-foreground">
            {{ profileBadges.length }}
          </span>
        </div>

        <template v-if="overview.profileBadgesAvailable">
          <p
            v-if="profileBadges.length === 0"
            class="rounded-lg border border-border/60 bg-secondary/30 p-4 text-sm text-muted-foreground"
          >
            {{ t('badges.profile_badges_empty') }}
          </p>
          <div v-else class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
            <div
              v-for="badge in profileBadges"
              :key="badge.id"
              class="flex items-center gap-3 rounded-lg border border-border/60 bg-card p-3"
            >
              <div
                class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-secondary/60 p-1.5"
              >
                <img
                  v-if="badgeIcon(badge)"
                  :src="badgeIcon(badge) ?? ''"
                  :alt="badge.description"
                  class="h-full w-full object-contain"
                  loading="lazy"
                />
                <span v-else class="text-sm font-semibold text-muted-foreground">
                  {{ initial(badge) }}
                </span>
              </div>
              <div class="min-w-0">
                <p class="truncate text-sm font-medium">{{ badge.description || badge.id }}</p>
                <p class="truncate text-xs text-muted-foreground">{{ badge.id }}</p>
              </div>
            </div>
          </div>
        </template>

        <template v-else>
          <p
            class="rounded-lg border border-border/60 bg-secondary/30 p-3 text-xs text-muted-foreground"
          >
            {{ t('badges.fallback_note') }}
          </p>
          <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
            <div
              v-for="badge in BADGE_CATALOG"
              :key="badge.id"
              class="flex items-center gap-3 rounded-lg border border-border/60 bg-card p-3 transition-colors"
              :class="earned.has(badge.id) ? '' : 'opacity-45'"
            >
              <img
                :src="badge.asset"
                :alt="badge.name"
                class="h-10 w-10 shrink-0 object-contain"
                loading="lazy"
              />
              <div class="min-w-0">
                <p class="truncate text-sm font-medium">{{ badge.name }}</p>
                <p class="text-xs text-muted-foreground">
                  {{ earned.has(badge.id) ? t('badges.earned') : t('badges.locked') }}
                </p>
              </div>
            </div>
          </div>
        </template>
      </section>

      <!-- Leveled tracks -->
      <section class="space-y-3">
        <h3 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          {{ t('badges.tracks_title') }}
        </h3>

        <div class="grid gap-4 lg:grid-cols-2">
          <!-- Account age -->
          <article class="rounded-xl border border-border/60 bg-card p-5">
            <div class="flex items-center justify-between gap-4">
              <div class="flex min-w-0 items-center gap-4">
                <div
                  class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-secondary/60 p-2"
                >
                  <img
                    v-if="currentTier"
                    :src="currentTier.asset"
                    :alt="currentTier.name"
                    class="h-full w-full object-contain"
                  />
                  <span v-else class="text-center text-[10px] leading-tight text-muted-foreground">
                    {{ t('badges.account_age.none') }}
                  </span>
                </div>
                <div class="min-w-0 space-y-0.5">
                  <p class="text-xs uppercase tracking-wide text-muted-foreground">
                    {{ t('badges.account_age.title') }}
                  </p>
                  <p class="truncate text-lg font-semibold">
                    {{ currentTier?.name ?? t('badges.account_age.none') }}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    {{ t('badges.account_age.created', { date: createdAt }) }}
                    · {{ t('badges.account_age.years', { count: ageYears }) }}
                  </p>
                </div>
              </div>
              <span class="shrink-0 text-sm font-medium text-muted-foreground">
                {{ accountAgePercent }}%
              </span>
            </div>

            <div class="mt-4 space-y-2">
              <Progress :model-value="accountAgePercent" class="h-2" />
              <p class="text-xs text-muted-foreground">
                <template v-if="nextTier">
                  {{
                    t('badges.account_age.next', {
                      name: nextTier.name,
                      years: nextTier.years,
                    })
                  }}
                </template>
                <template v-else>{{ t('badges.account_age.maxed') }}</template>
              </p>
            </div>

            <div class="mt-4 flex flex-wrap gap-2">
              <div
                v-for="tier in ACCOUNT_AGE_TIERS"
                :key="tier.level"
                class="flex items-center gap-1.5 rounded-full border border-border/60 px-2 py-1 text-xs"
                :class="
                  (overview.accountAgeLevel ?? 0) >= tier.level
                    ? 'bg-secondary/50 text-foreground'
                    : 'opacity-40'
                "
              >
                <img :src="tier.asset" :alt="tier.name" class="h-4 w-4 object-contain" />
                <span>{{ tier.name }}</span>
              </div>
            </div>
          </article>

          <!-- Game Time -->
          <article class="rounded-xl border border-border/60 bg-card p-5">
            <div class="flex items-center justify-between gap-4">
              <div class="flex min-w-0 items-center gap-4">
                <div
                  class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-secondary/60 p-2"
                >
                  <img
                    v-if="gameTimeTier"
                    :src="gameTimeTier.asset"
                    :alt="gameTimeTier.name"
                    class="h-full w-full object-contain"
                  />
                  <span v-else class="text-center text-[10px] leading-tight text-muted-foreground">
                    {{ hoursLabel(0) }}
                  </span>
                </div>
                <div class="min-w-0 space-y-0.5">
                  <p class="text-xs uppercase tracking-wide text-muted-foreground">Game Time</p>
                  <p class="truncate text-lg font-semibold">
                    {{ gameTimeTier?.name ?? t('badges.not_reached') }}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    {{ hoursLabel(gameTimeHours) }}
                    <template v-if="overview.gameTimeLevel > 0">
                      · {{ t('badges.level', { level: overview.gameTimeLevel }) }}
                    </template>
                  </p>
                </div>
              </div>
              <span class="shrink-0 text-sm font-medium text-muted-foreground">
                {{ gameTimePercent }}%
              </span>
            </div>

            <div class="mt-4 space-y-2">
              <Progress :model-value="gameTimePercent" class="h-2" />
              <p class="text-xs text-muted-foreground">
                <template v-if="gameTimeNext">
                  {{ nextLabel(gameTimeNext, 'hours') }}
                </template>
                <template v-else>{{ t('badges.maxed') }}</template>
              </p>
            </div>

            <p class="mt-2 text-[11px] leading-relaxed text-muted-foreground/80">
              {{ t('badges.game_time_source') }}
            </p>

            <div class="mt-3 flex flex-wrap gap-2">
              <div
                v-for="tier in GAME_TIME_TIERS"
                :key="tier.level"
                class="flex items-center gap-1.5 rounded-full border border-border/60 px-2 py-1 text-xs"
                :class="
                  (overview.gameTimeLevel ?? 0) >= tier.level
                    ? 'bg-secondary/50 text-foreground'
                    : 'opacity-40'
                "
              >
                <img :src="tier.asset" :alt="tier.name" class="h-4 w-4 object-contain" />
                <span>{{ tier.name }}</span>
              </div>
            </div>
          </article>

          <!-- Game Variety -->
          <article class="rounded-xl border border-border/60 bg-card p-5">
            <div class="flex items-center justify-between gap-4">
              <div class="flex min-w-0 items-center gap-4">
                <div
                  class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-secondary/60 p-2"
                >
                  <img
                    v-if="gameVarietyTier"
                    :src="gameVarietyTier.asset"
                    :alt="gameVarietyTier.name"
                    class="h-full w-full object-contain"
                  />
                  <span v-else class="text-center text-[10px] leading-tight text-muted-foreground">
                    {{ gamesLabel(0) }}
                  </span>
                </div>
                <div class="min-w-0 space-y-0.5">
                  <p class="text-xs uppercase tracking-wide text-muted-foreground">Game Variety</p>
                  <p class="truncate text-lg font-semibold">
                    {{ gameVarietyTier?.name ?? t('badges.not_reached') }}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    {{ gamesLabel(gameVarietyCount) }}
                    <template v-if="overview.gameVarietyLevel > 0">
                      · {{ t('badges.level', { level: overview.gameVarietyLevel }) }}
                    </template>
                  </p>
                </div>
              </div>
              <span class="shrink-0 text-sm font-medium text-muted-foreground">
                {{ gameVarietyPercent }}%
              </span>
            </div>

            <div class="mt-4 space-y-2">
              <Progress :model-value="gameVarietyPercent" class="h-2" />
              <p class="text-xs text-muted-foreground">
                <template v-if="gameVarietyNext">
                  {{ nextLabel(gameVarietyNext, 'games') }}
                </template>
                <template v-else>{{ t('badges.maxed') }}</template>
              </p>
            </div>

            <p class="mt-2 text-[11px] leading-relaxed text-muted-foreground/80">
              {{ t('badges.game_variety_source') }}
            </p>

            <div class="mt-3 flex flex-wrap gap-2">
              <div
                v-for="tier in GAME_VARIETY_TIERS"
                :key="tier.level"
                class="flex items-center gap-1.5 rounded-full border border-border/60 px-2 py-1 text-xs"
                :class="
                  (overview.gameVarietyLevel ?? 0) >= tier.level
                    ? 'bg-secondary/50 text-foreground'
                    : 'opacity-40'
                "
              >
                <img :src="tier.asset" :alt="tier.name" class="h-4 w-4 object-contain" />
                <span>{{ tier.name }}</span>
              </div>
            </div>
          </article>
        </div>
      </section>

      <!-- Tracks Discord does not expose -->
      <section class="space-y-3">
        <h3 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          {{ t('badges.unavailable_title') }}
        </h3>

        <div class="grid gap-4 lg:grid-cols-2">
          <article class="rounded-xl border border-border/60 bg-card p-5">
            <p class="text-xs uppercase tracking-wide text-muted-foreground">Streaming</p>
            <p class="mt-1 text-[11px] leading-relaxed text-muted-foreground/80">
              {{ t('badges.streaming_note') }}
            </p>
            <div class="mt-3 flex flex-wrap gap-2">
              <div
                v-for="tier in STREAMING_TIERS"
                :key="tier.level"
                class="flex items-center gap-1.5 rounded-full border border-border/60 px-2 py-1 text-xs"
              >
                <img :src="tier.asset" :alt="tier.name" class="h-4 w-4 object-contain" />
                <span>{{ tier.name }}</span>
              </div>
            </div>
          </article>

          <article class="rounded-xl border border-border/60 bg-card p-5">
            <p class="text-xs uppercase tracking-wide text-muted-foreground">Gifting</p>
            <p class="mt-1 text-[11px] leading-relaxed text-muted-foreground/80">
              {{ t('badges.gifting_note') }}
            </p>
            <div class="mt-3 flex flex-wrap gap-2">
              <div
                v-for="tier in GIFTING_TIERS"
                :key="tier.level"
                class="flex items-center gap-1.5 rounded-full border border-border/60 px-2 py-1 text-xs"
              >
                <img :src="tier.asset" :alt="tier.name" class="h-4 w-4 object-contain" />
                <span>{{ tier.name }}</span>
              </div>
            </div>
          </article>
        </div>
      </section>

      <p
        class="rounded-lg border border-border/60 bg-secondary/30 p-4 text-xs leading-relaxed text-muted-foreground"
      >
        {{ t('badges.api_note') }}
      </p>
    </template>
  </div>
</template>
