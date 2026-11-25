/**
 * Runtime platform detection for the UI.
 *
 * Tauri injects `import.meta.env.TAURI_ENV_*` at build time, but the mobile and
 * desktop bundles are produced by different builds, and the desktop bundle is
 * also opened in a plain browser during development. The user agent is the one
 * signal that is correct in every case, and it is already the pattern used by
 * `DesktopClientPicker.vue`.
 *
 * Used to hide desktop-only chrome (the frameless title bar, the CDP exit
 * dialog) that would otherwise call window APIs Android does not implement.
 */
export function isMobilePlatform(): boolean {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent
  return /Android|iPhone|iPad|iPod/i.test(ua)
}
