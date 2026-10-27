import { useState } from 'react';
import { Settings, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { useSettingsStore, type Language } from '@/stores/settings';
import { setAutoLockMinutes as persistAutoLockMinutes } from '@/lib/tauri';
import { useI18n } from '@/i18n';

const selectClass =
  'w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100';

export function SettingsModal() {
  const { t, language } = useI18n();
  const [isOpen, setIsOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const autoLockMinutes = useSettingsStore((s) => s.autoLockMinutes);
  const clipboardClearSeconds = useSettingsStore((s) => s.clipboardClearSeconds);
  const setAutoLockMinutes = useSettingsStore((s) => s.setAutoLockMinutes);
  const setClipboardClearSeconds = useSettingsStore((s) => s.setClipboardClearSeconds);
  const setLanguage = useSettingsStore((s) => s.setLanguage);

  const autoLockOptions = [
    { value: 0, label: t('settings.off') },
    { value: 1, label: t('settings.minute') },
    { value: 5, label: t('settings.minutes', { count: 5 }) },
    { value: 15, label: t('settings.minutes', { count: 15 }) },
    { value: 30, label: t('settings.minutes', { count: 30 }) },
  ];

  const clipboardOptions = [
    { value: 15, label: t('settings.seconds', { count: 15 }) },
    { value: 30, label: t('settings.seconds', { count: 30 }) },
    { value: 60, label: t('settings.seconds', { count: 60 }) },
    { value: 0, label: t('settings.neverClear') },
  ];

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
        <Settings className="mr-2 h-4 w-4" /> {t('settings.open')}
      </Button>

      <Dialog open={isOpen} onClose={() => setIsOpen(false)} title={t('settings.title')}>
        <div className="space-y-5">
          {error && (
            <div className="flex items-center gap-2 rounded-md bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
              <AlertCircle className="h-4 w-4 shrink-0" />
              {error}
            </div>
          )}

          <div>
            <label className="mb-1 block text-sm font-medium">{t('settings.language')}</label>
            <select
              className={selectClass}
              value={language}
              onChange={(e) => setLanguage(e.target.value as Language)}
            >
              <option value="en">{t('settings.languageEn')}</option>
              <option value="zh">{t('settings.languageZh')}</option>
            </select>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium">{t('settings.autoLock')}</label>
            <select
              className={selectClass}
              value={autoLockMinutes}
              onChange={(e) => void handleAutoLockChange(Number(e.target.value))}
            >
              {autoLockOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              {t('settings.autoLockHint')}
            </p>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium">{t('settings.clipboard')}</label>
            <select
              className={selectClass}
              value={clipboardClearSeconds}
              onChange={(e) => setClipboardClearSeconds(Number(e.target.value))}
            >
              {clipboardOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              {t('settings.clipboardHint')}
            </p>
          </div>

          <div className="flex justify-end">
            <Button onClick={() => setIsOpen(false)}>{t('settings.done')}</Button>
          </div>
        </div>
      </Dialog>
    </>
  );
}
