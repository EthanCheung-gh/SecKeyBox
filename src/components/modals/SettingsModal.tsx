import { useState } from 'react';
import { Settings, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { useSettingsStore } from '@/stores/settings';
import { setAutoLockMinutes as persistAutoLockMinutes } from '@/lib/tauri';

const AUTO_LOCK_OPTIONS = [
  { value: 0, label: 'Off' },
  { value: 1, label: '1 minute' },
  { value: 5, label: '5 minutes' },
  { value: 15, label: '15 minutes' },
  { value: 30, label: '30 minutes' },
];

const CLIPBOARD_OPTIONS = [
  { value: 15, label: '15 seconds' },
  { value: 30, label: '30 seconds' },
  { value: 60, label: '60 seconds' },
  { value: 0, label: 'Never clear' },
];

const selectClass =
  'w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100';

export function SettingsModal() {
  const [isOpen, setIsOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const autoLockMinutes = useSettingsStore((s) => s.autoLockMinutes);
  const clipboardClearSeconds = useSettingsStore((s) => s.clipboardClearSeconds);
  const setAutoLockMinutes = useSettingsStore((s) => s.setAutoLockMinutes);
  const setClipboardClearSeconds = useSettingsStore((s) => s.setClipboardClearSeconds);

  const handleAutoLockChange = async (minutes: number) => {
    const previous = autoLockMinutes;
    setAutoLockMinutes(minutes);
    setError(null);
    try {
      await persistAutoLockMinutes(minutes);
    } catch (e) {
      setAutoLockMinutes(previous);
      setError(String(e));
    }
  };

  return (
    <>
      <Button
        variant="ghost"
        size="sm"
        className="w-full justify-start text-gray-500 dark:text-gray-400"
        onClick={() => setIsOpen(true)}
      >
        <Settings className="mr-2 h-4 w-4" /> Settings
      </Button>

      <Dialog open={isOpen} onClose={() => setIsOpen(false)} title="Security Settings">
        <div className="space-y-5">
          {error && (
            <div className="flex items-center gap-2 rounded-md bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
              <AlertCircle className="h-4 w-4 shrink-0" />
              {error}
            </div>
          )}

          <div>
            <label className="mb-1 block text-sm font-medium">Auto-lock after inactivity</label>
            <select
              className={selectClass}
              value={autoLockMinutes}
              onChange={(e) => void handleAutoLockChange(Number(e.target.value))}
            >
              {AUTO_LOCK_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Enforced by the app process, so it also applies while the window is hidden.
            </p>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium">Clear clipboard after copying</label>
            <select
              className={selectClass}
              value={clipboardClearSeconds}
              onChange={(e) => setClipboardClearSeconds(Number(e.target.value))}
            >
              {CLIPBOARD_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              The clipboard is always cleared when the vault locks or the app exits.
            </p>
          </div>

          <div className="flex justify-end">
            <Button onClick={() => setIsOpen(false)}>Done</Button>
          </div>
        </div>
      </Dialog>
    </>
  );
}
