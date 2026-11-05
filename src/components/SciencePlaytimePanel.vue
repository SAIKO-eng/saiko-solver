<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ShieldCheck, Timer } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import SettingsSectionCard from '@/components/settings/SettingsSectionCard.vue'
import { useQuestsStore } from '@/stores/quests'
import { commandErrorMessage } from '@/utils/commandError'

interface SciencePlaytimeStatus {
  running: boolean
  total: number
  processed: number
  failed: number
  hours: number
  started_at_ms: number | null
  finished_at_ms: number | null
  last_server_code: number | null
  last_error: string | null
}

const emptyStatus: SciencePlaytimeStatus = {
  running: false,
  total: 0,
  processed: 0,
  failed: 0,
  hours: 0,
  started_at_ms: null,
  finished_at_ms: null,
  last_server_code: null,
  last_error: null,
}

const hours = ref(1)
const gamesLimit = ref<number | null>(null)
const status = ref<SciencePlaytimeStatus>({ ...emptyStatus })
const busy = ref(false)
const error = ref<string | null>(null)
const questsStore = useQuestsStore()

let timer: ReturnType<typeof setInterval> | null = null

const percent = computed(() => {
  if (!status.value.total) {
    return 0
  }
  return Math.min(100, Math.round((status.value.processed / status.value.total) * 100))
})

const summary = computed(() => {
  const current = status.value
  if (current.running) {
    return `${current.processed}/${current.total} games · ${current.hours}h each`
  }
  if (current.total === 0) {
    return 'Idle'
  }
  return `${current.processed}/${current.total} games sent · ${current.failed} failed`
})

function startPolling() {
  if (timer) {
    return
  }
  timer = setInterval(refresh, 1000)
}

function stopPolling() {
  if (timer) {
    clearInterval(timer)
    timer = null
  }
}

async function refresh() {
  try {
    status.value = await invoke<SciencePlaytimeStatus>('get_science_playtime_status')
  } catch (cause) {
    error.value = commandErrorMessage(cause)
    stopPolling()
  }
}

async function start() {
  if (questsStore.safeMode) {
    error.value = 'Safe Mode is enabled. Disable it in Settings → Quest Behavior before starting.'
    return
  }
  busy.value = true
  error.value = null
  try {
    await invoke('start_science_playtime', {
      hours: Number(hours.value) || 0,
      gamesLimit: gamesLimit.value && gamesLimit.value > 0 ? gamesLimit.value : null,
    })
    await refresh()
    startPolling()
  } catch (cause) {
    error.value = commandErrorMessage(cause)
  } finally {
    busy.value = false
  }
}

async function cancel() {
  busy.value = true
  try {
    await invoke('cancel_science_playtime')
    await refresh()
  } catch (cause) {
    error.value = commandErrorMessage(cause)
  } finally {
    busy.value = false
  }
}

onMounted(async () => {
  await refresh()
  if (status.value.running) {
    if (questsStore.safeMode) {
      await cancel()
    } else {
      startPolling()
    }
  }
})

// Enabling Safe Mode while an injection is running must stop it, not just
// disable the button for the next run.
watch(() => questsStore.safeMode, (enabled) => {
  if (enabled && status.value.running) {
    void cancel()
  }
})

onBeforeUnmount(stopPolling)
</script>

<template>
  <SettingsSectionCard
    title="Science Playtime"
    description="Injects launch and heartbeat analytics events to credit Game Time / Games Played."
    :icon="Timer"
  >
    <div class="space-y-4">
      <div
        v-if="questsStore.safeMode"
        class="flex items-start gap-2.5 rounded-lg border border-amber-500/25 bg-amber-500/5 px-4 py-3 text-sm"
      >
        <ShieldCheck class="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
        <div class="min-w-0">
          <p class="font-medium">Disabled by Safe Mode</p>
          <p class="mt-1 text-muted-foreground">
            Science playtime injection is turned off because it fabricates activity Discord can detect.
            Turn off Safe Mode in Settings → Quest Behavior only if you accept the account risk.
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <label class="space-y-1.5 text-sm">
          <span class="font-medium">Hours per game</span>
          <input
            v-model.number="hours"
            type="number"
            min="0"
            step="0.5"
            :disabled="status.running || questsStore.safeMode"
            class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm disabled:opacity-50"
          />
        </label>

        <label class="space-y-1.5 text-sm">
          <span class="font-medium">Game limit</span>
          <input
            v-model.number="gamesLimit"
            type="number"
            min="1"
            placeholder="all detectable games"
            :disabled="status.running || questsStore.safeMode"
            class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm disabled:opacity-50"
          />
        </label>
      </div>

      <div class="space-y-2">
        <div class="flex items-center justify-between text-sm">
          <span>{{ summary }}</span>
          <span v-if="status.total > 0" class="text-muted-foreground">{{ percent }}%</span>
        </div>
        <Progress :model-value="percent" />
      </div>

      <p
        v-if="status.last_error"
        class="text-sm text-destructive"
      >
        {{ status.last_error }}
      </p>
      <p
        v-else-if="error"
        class="text-sm text-destructive"
      >
        {{ error }}
      </p>
      <p
        v-else-if="status.last_server_code"
        class="text-sm text-muted-foreground"
      >
        Last server response: HTTP {{ status.last_server_code }}
      </p>

      <div class="flex gap-2">
        <Button
          v-if="!status.running"
          :disabled="busy || questsStore.safeMode"
          @click="start"
        >
          Start
        </Button>
        <Button
          v-else
          variant="destructive"
          :disabled="busy"
          @click="cancel"
        >
          Stop
        </Button>
      </div>
    </div>
  </SettingsSectionCard>
</template>
