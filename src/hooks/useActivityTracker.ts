import { useEffect, useRef } from 'react';
import { useVaultStore } from '@/stores/vault';
import { useSettingsStore } from '@/stores/settings';
import { getSecuritySettings, recordActivity } from '@/lib/tauri';

const REPORT_INTERVAL_MS = 10_000;

/**
 * Idle-lock tracking with two layers:
 *
 * 1. The Rust watchdog is authoritative: it locks the vault and clears the
 *    clipboard even when the webview is hidden or its timers are throttled.
 *    Here we only report activity (throttled to one call per 10s).
 * 2. A local timer mirrors the policy so the UI switches to the unlock screen
 *    immediately instead of waiting for the next watchdog tick.
 */
export function useActivityTracker() {
  const isUnlocked = useVaultStore((s) => s.isUnlocked);
  const autoLockMinutes = useSettingsStore((s) => s.autoLockMinutes);
  const setAutoLockMinutes = useSettingsStore((s) => s.setAutoLockMinutes);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Pull the authoritative policy from the backend when the vault unlocks.
  useEffect(() => {
    if (!isUnlocked) return;
    let cancelled = false;
    getSecuritySettings()
      .then((settings) => {
        if (!cancelled) setAutoLockMinutes(settings.auto_lock_minutes);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [isUnlocked, setAutoLockMinutes]);

  useEffect(() => {
    if (!isUnlocked) return;

    let lastReport = 0;
    const report = () => {
      const now = Date.now();
      if (now - lastReport < REPORT_INTERVAL_MS) return;
      lastReport = now;
      void recordActivity().catch(() => undefined);
    };

    const armLocalTimer = () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      if (autoLockMinutes <= 0) return;
      timerRef.current = setTimeout(() => {
        void useVaultStore.getState().lock();
      }, autoLockMinutes * 60 * 1000);
    };

    const handleActivity = () => {
      report();
      armLocalTimer();
    };

    const events = ['mousedown', 'keydown', 'touchstart', 'wheel'] as const;
    events.forEach((event) => window.addEventListener(event, handleActivity, { passive: true }));
    window.addEventListener('focus', handleActivity);

    report();
    armLocalTimer();

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      events.forEach((event) => window.removeEventListener(event, handleActivity));
      window.removeEventListener('focus', handleActivity);
    };
  }, [isUnlocked, autoLockMinutes]);
}
