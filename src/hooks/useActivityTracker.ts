import { useEffect, useRef } from 'react';
import { useVaultStore } from '@/stores/vault';

const INACTIVITY_TIMEOUT = 5 * 60 * 1000; // 5 minutes

export function useActivityTracker() {
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const isUnlocked = useVaultStore((s) => s.isUnlocked);
  const lock = useVaultStore((s) => s.lock);

  useEffect(() => {
    if (!isUnlocked) return;

    const resetTimer = () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      timerRef.current = setTimeout(() => {
        lock();
      }, INACTIVITY_TIMEOUT);
    };

    const events = ['mousedown', 'keydown', 'touchstart'];
    events.forEach((event) => {
      window.addEventListener(event, resetTimer);
    });

    resetTimer();

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      events.forEach((event) => {
        window.removeEventListener(event, resetTimer);
      });
    };
  }, [isUnlocked, lock]);
}