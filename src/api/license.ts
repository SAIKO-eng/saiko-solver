import { invoke } from '@tauri-apps/api/core'

export interface LicenseStatus {
  state: 'unactivated' | 'active' | 'expired' | 'invalid'
  machine_id: string
  expires_at: number | null
  plan: string
  message: string
}

export function getLicenseStatus(): Promise<LicenseStatus> {
  return invoke('license::get_license_status')
}

export function getMachineId(): Promise<string> {
  return invoke('license::get_machine_id')
}

export function activateLicense(code: string): Promise<LicenseStatus> {
  return invoke('license::activate_license', { code })
}