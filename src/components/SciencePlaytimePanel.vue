<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Timer } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import SettingsSectionCard from '@/components/settings/SettingsSectionCard.vue'
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
    startPolling()
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
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <label class="space-y-1.5 text-sm">
          <span class="font-medium">Hours per game</span>
          <input
            v-model.number="hours"
            type="number"
            min="0"
            step="0.5"
            :disabled="status.running"
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
            :disabled="status.running"
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
          :disabled="busy"
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
