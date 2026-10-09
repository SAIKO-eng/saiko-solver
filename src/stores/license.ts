import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import {
  activateLicense as apiActivateLicense,
  getLicenseStatus,
  type LicenseStatus,
} from '@/api/license'

export const useLicenseStore = defineStore('license', () => {
  const status = ref<LicenseStatus | null>(null)
  const loading = ref(false)
  const lastError = ref('')

  const initialized = computed(() => status.value !== null || loading.value)
  const activated = computed(() => status.value?.state === 'active')

  async function initialize() {
    if (status.value) return
    loading.value = true
    lastError.value = ''
    try {
      status.value = await getLicenseStatus()
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : String(error)
    } finally {
      loading.value = false
    }
  }

  async function activate(code: string) {
    lastError.value = ''
    loading.value = true
    try {
      status.value = await apiActivateLicense(code.trim())
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : String(error)
      throw error
    } finally {
      loading.value = false
    }
  }

  return {
    status,
    loading,
    lastError,
    initialized,
    activated,
    initialize,
    activate,
  }
})