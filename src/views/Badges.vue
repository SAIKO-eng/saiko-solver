<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RefreshCw } from 'lucide-vue-next'
import { getBadgeOverview, type BadgeOverview } from '@/api/badges'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { ACCOUNT_AGE_TIERS, BADGE_CATALOG, accountAgeTier } from '@/utils/badgeCatalog'

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
const progressPercent = computed(() =>
  Math.round((overview.value?.accountAgeProgress ?? 0) * 100),
)
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
      <!-- Account age -->
      <section class="rounded-xl border border-border/60 bg-card p-5">
        <div class="flex items-center justify-between gap-4">
          <div class="flex min-w-0 items-center gap-4">
            <div
              class="flex h-16 w-16 shrink-0 items-center justify-center rounded-full bg-secondary/60 p-2"
            >
              <img
                v-if="currentTier"
                :src="currentTier.asset"
                :alt="currentTier.name"
                class="h-full w-full object-contain"
              />
              <span v-else class="text-xs text-muted-foreground">{{ t('badges.account_age.none') }}</span>
            </div>
            <div class="min-w-0 space-y-1">
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
            {{ progressPercent }}%
          </span>
        </div>

        <div class="mt-4 space-y-2">
          <Progress :model-value="progressPercent" class="h-2" />
          <p class="text-xs text-muted-foreground">
            <template v-if="nextTier">
              {{ t('badges.account_age.next', { name: nextTier.name, years: nextTier.years }) }}
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
      </section>

      <!-- Badge grid -->
      <section class="space-y-3">
        <h3 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          {{ t('badges.profile_badges') }}
        </h3>
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
      </section>

      <p class="rounded-lg border border-border/60 bg-secondary/30 p-4 text-xs leading-relaxed text-muted-foreground">
        {{ t('badges.api_note') }}
      </p>
    </template>
  </div>
</template>
