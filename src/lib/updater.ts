import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdateStatus =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'up-to-date' }
  | { kind: 'available'; version: string; notes?: string }
  | { kind: 'downloading'; progress: number }
  | { kind: 'ready' }
  | { kind: 'error'; message: string };

/**
 * Check the configured endpoint for a newer signed release.
 * Returns null when the app is up to date.
 */
export async function checkForUpdate(): Promise<Update | null> {
  return check();
}

/**
 * Download and install a pending update, reporting progress in [0, 1].
 * The caller decides when to relaunch.
 */
export async function installUpdate(
  update: Update,
  onProgress: (progress: number) => void
): Promise<void> {
  let total = 0;
  let downloaded = 0;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case 'Started':
        total = event.data.contentLength ?? 0;
        onProgress(0);
        break;
      case 'Progress':
        downloaded += event.data.chunkLength;
        onProgress(total > 0 ? Math.min(downloaded / total, 1) : 0);
        break;
      case 'Finished':
        onProgress(1);
        break;
    }
  });
}

/** Restart the app so the installed update takes effect. */
export async function restartApp(): Promise<void> {
  await relaunch();
}
