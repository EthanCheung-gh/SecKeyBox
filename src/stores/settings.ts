import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type Theme = 'light' | 'dark' | 'system';

interface SettingsState {
  theme: Theme;
  /** Idle minutes before the vault locks; 0 disables. Mirrors the Rust setting. */
  autoLockMinutes: number;
  /** Seconds before the clipboard is cleared after a copy; 0 keeps it. */
  clipboardClearSeconds: number;
  setTheme: (theme: Theme) => void;
  setAutoLockMinutes: (minutes: number) => void;
  setClipboardClearSeconds: (seconds: number) => void;
}

function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  const effectiveTheme = theme === 'system' ? getSystemTheme() : theme;
  
  if (effectiveTheme === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      theme: 'system',
      autoLockMinutes: 5,
      clipboardClearSeconds: 30,
      
      setTheme: (theme) => {
        set({ theme });
        applyTheme(theme);
      },

      setAutoLockMinutes: (autoLockMinutes) => set({ autoLockMinutes }),

      setClipboardClearSeconds: (clipboardClearSeconds) => set({ clipboardClearSeconds }),
    }),
    {
      name: 'seckeybox-settings',
      onRehydrateStorage: () => (state) => {
        if (state) {
          applyTheme(state.theme);
        }
      },
    }
  )
);

// Initialize theme on load
if (typeof window !== 'undefined') {
  const theme = useSettingsStore.getState().theme;
  applyTheme(theme);
  
  // Listen for system theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const currentTheme = useSettingsStore.getState().theme;
    if (currentTheme === 'system') {
      applyTheme('system');
    }
  });
}
