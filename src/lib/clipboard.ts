import { clearClipboard, copyToClipboard } from '@/lib/tauri';
import { useSettingsStore } from '@/stores/settings';

/**
 * Copy a secret and arm the clipboard auto-clear timer.
 *
 * Every copy site goes through here so the configured policy
 * (`clipboardClearSeconds`, 0 = keep) is applied consistently.
 */
export async function copySecret(text: string): Promise<void> {
  await copyToClipboard(text);
  const seconds = useSettingsStore.getState().clipboardClearSeconds;
  if (seconds <= 0) return;
  // Don't swallow the promise: the copy already succeeded, and a failed
  // clear must not surface as a copy failure.
  window.setTimeout(() => {
    void clearClipboard().catch(() => undefined);
  }, seconds * 1000);
}
