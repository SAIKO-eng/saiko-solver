<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Check, Copy, KeyRound, Loader2, ShieldCheck, TriangleAlert } from 'lucide-vue-next'
import { useLicenseStore } from '@/stores/license'
import { getMachineId } from '@/api/license'

const { t } = useI18n()
const licenseStore = useLicenseStore()

const machineId = ref('')
const code = ref('')
const loadingMachineId = ref(true)
const copied = ref(false)

const storedProblem = computed(() => {
  const state = licenseStore.status?.state
  if (state === 'expired') return 'license.expired_stored'
  if (state === 'invalid') return 'license.invalid_stored'
  return ''
})

const expiresLabel = computed(() => {
  const expiry = licenseStore.status?.expires_at
  if (!expiry) return t('license.lifetime')
  const date = new Date(expiry * 1000)
  return isNaN(date.getTime()) ? t('license.lifetime') : date.toLocaleDateString()
})

function errorMessageKey(message: string): string {
  if (!message) return 'license.error.unknown'
  if (message.includes('different machine')) return 'license.error.machine'
  if (message.includes('expired')) return 'license.error.expired'
  if (message.includes('signature') || message.includes('Malformed') || message.includes('Unsupported')) {
    return 'license.error.signature'
  }
  return 'license.error.unknown'
}

async function copyMachineId() {
  try {
    await navigator.clipboard.writeText(machineId.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch {
    // clipboard unavailable
  }
}

async function submit() {
  if (!code.value.trim() || licenseStore.loading) return
  await licenseStore.activate(code.value)
}

onMounted(async () => {
  licenseStore.initialize()
  try {
    machineId.value = await getMachineId()
  } finally {
    loadingMachineId.value = false
  }
})
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center bg-background text-foreground p-6">
    <div
      class="w-full max-w-md rounded-2xl border bg-card/90 p-8 shadow-xl backdrop-blur-md"
    >
      <div class="flex flex-col items-center text-center">
        <img src="/icons/logo.png" :alt="t('general.title')" class="h-14 w-14" />
        <h1 class="mt-4 text-2xl font-semibold tracking-tight">
          {{ t('general.title') }}
        </h1>
        <p class="mt-1 text-sm text-muted-foreground">
          {{ t('license.subtitle') }}
        </p>
      </div>

      <div v-if="licenseStore.activated" class="mt-6 space-y-4">
        <div class="flex items-center justify-center gap-2 rounded-xl border border-green-500/30 bg-green-500/10 py-4">
          <ShieldCheck class="h-5 w-5 text-green-500" />
          <span class="text-sm font-medium text-green-600 dark:text-green-400">
            {{ t('license.activated') }}
          </span>
        </div>

        <div class="flex items-center justify-between rounded-xl border px-4 py-3 text-sm">
          <span class="text-muted-foreground">{{ t('license.plan') }}</span>
          <span class="font-medium">{{ licenseStore.status?.plan || '-' }}</span>
        </div>

        <div class="flex items-center justify-between rounded-xl border px-4 py-3 text-sm">
          <span class="text-muted-foreground">{{ t('license.expires') }}</span>
          <span class="font-medium">{{ expiresLabel }}</span>
        </div>
      </div>

      <form v-else class="mt-6 space-y-4" @submit.prevent="submit">
        <div v-if="storedProblem" class="flex items-start gap-2 rounded-xl border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-600 dark:text-amber-400">
          <TriangleAlert class="mt-0.5 h-4 w-4 shrink-0" />
          <span>{{ t(storedProblem) }}</span>
        </div>

        <div class="space-y-1.5">
          <Label>{{ t('license.machine_id') }}</Label>
          <div class="flex items-center gap-2">
            <Input
              :model-value="machineId"
              readonly
              :disabled="loadingMachineId"
              class="font-mono text-xs"
            />
            <Button
              type="button"
              variant="outline"
              size="icon"
              class="h-9 w-9 shrink-0"
              :title="t('license.copy')"
              :disabled="loadingMachineId || !machineId"
              @click="copyMachineId"
            >
              <Check v-if="copied" class="h-4 w-4 text-green-500" />
              <Copy v-else class="h-4 w-4" />
            </Button>
          </div>
          <p class="text-xs text-muted-foreground">
            {{ t('license.machine_id_hint') }}
          </p>
        </div>

        <div class="space-y-1.5">
          <Label>{{ t('license.code') }}</Label>
          <Input
            v-model="code"
            :placeholder="t('license.code_placeholder')"
            autocomplete="off"
            spellcheck="false"
            :disabled="licenseStore.loading"
            class="font-mono text-sm"
          />
        </div>

        <div
          v-if="licenseStore.lastError"
          class="flex items-start gap-2 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-600 dark:text-red-400"
        >
          <TriangleAlert class="mt-0.5 h-4 w-4 shrink-0" />
          <span>{{ t(errorMessageKey(licenseStore.lastError)) }}</span>
        </div>

        <Button class="w-full" type="submit" :disabled="licenseStore.loading || !code.trim()">
          <Loader2 v-if="licenseStore.loading" class="mr-2 h-4 w-4 animate-spin" />
          <KeyRound v-else class="mr-2 h-4 w-4" />
          {{ licenseStore.loading ? t('license.activating') : t('license.activate') }}
        </Button>
      </form>

      <p class="mt-6 text-center text-xs text-muted-foreground">
        {{ t('license.help_footer') }}
      </p>
    </div>
  </div>
</template>