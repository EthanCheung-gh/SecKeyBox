import { useEffect, useState } from 'react';
import type { Update } from '@tauri-apps/plugin-updater';
import { Settings, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { useSettingsStore, type Language } from '@/stores/settings';
import {
  setAutoLockMinutes as persistAutoLockMinutes,
  createBackup,
  listBackups,
  deleteBackup,
  restoreBackup,
} from '@/lib/tauri';
import { useVaultStore } from '@/stores/vault';
import type { BackupInfo } from '@/types';
import { useI18n } from '@/i18n';
import { checkForUpdate, installUpdate, restartApp } from '@/lib/updater';

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

  // --- updates ---
  const [updateState, setUpdateState] = useState<
    | { kind: 'idle' }
    | { kind: 'checking' }
    | { kind: 'up-to-date' }
    | { kind: 'available'; update: Update }
    | { kind: 'downloading'; percent: number }
    | { kind: 'ready' }
    | { kind: 'error'; message: string }
  >({ kind: 'idle' });

  const handleCheckUpdate = async () => {
    setUpdateState({ kind: 'checking' });
    try {
      const update = await checkForUpdate();
      setUpdateState(update ? { kind: 'available', update } : { kind: 'up-to-date' });
    } catch (e) {
      setUpdateState({ kind: 'error', message: e instanceof Error ? e.message : String(e) });
    }
  };

  const handleInstall = async (update: Update) => {
    setUpdateState({ kind: 'downloading', percent: 0 });
    try {
      await installUpdate(update, (progress) =>
        setUpdateState({ kind: 'downloading', percent: Math.round(progress * 100) })
      );
      setUpdateState({ kind: 'ready' });
    } catch (e) {
      setUpdateState({ kind: 'error', message: e instanceof Error ? e.message : String(e) });
    }
  };

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

  // --- backups ---
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [backupBusy, setBackupBusy] = useState(false);
  const [backupNotice, setBackupNotice] = useState<string | null>(null);

  const refreshBackups = async () => {
    try {
      setBackups(await listBackups());
    } catch {
      setBackups([]);
    }
  };

  useEffect(() => {
    if (isOpen) {
      void refreshBackups();
      setBackupNotice(null);
    }
  }, [isOpen]);

  const formatSize = (bytes: number) =>
    bytes >= 1024 * 1024 ? `${(bytes / 1024 / 1024).toFixed(1)} MB` : `${Math.round(bytes / 1024)} KB`;

  const handleCreateBackup = async () => {
    setBackupBusy(true);
    setBackupNotice(null);
    try {
      await createBackup();
      await refreshBackups();
    } catch (e) {
      setBackupNotice(t('backup.error', { message: e instanceof Error ? e.message : String(e) }));
    } finally {
      setBackupBusy(false);
    }
  };

  const handleRestore = async (backup: BackupInfo) => {
    const time = new Date(backup.created_at * 1000).toLocaleString();
    if (!window.confirm(t('backup.restoreConfirm', { time }))) return;
    setBackupBusy(true);
    setBackupNotice(null);
    try {
      await restoreBackup(backup.file);
      setBackupNotice(t('backup.restored'));
      await useVaultStore.getState().lock();
      setIsOpen(false);
    } catch (e) {
      setBackupNotice(t('backup.error', { message: e instanceof Error ? e.message : String(e) }));
    } finally {
      setBackupBusy(false);
    }
  };

  const handleDeleteBackup = async (backup: BackupInfo) => {
    setBackupBusy(true);
    try {
      await deleteBackup(backup.file);
      await refreshBackups();
    } catch (e) {
      setBackupNotice(t('backup.error', { message: e instanceof Error ? e.message : String(e) }));
    } finally {
      setBackupBusy(false);
    }
  };

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

          <div>
            <label className="mb-1 block text-sm font-medium">{t('backup.title')}</label>
            <div className="space-y-2 rounded-md border border-gray-200 p-3 dark:border-gray-700">
              <p className="text-xs text-gray-500 dark:text-gray-400">{t('backup.hint')}</p>
              <Button
                variant="outline"
                size="sm"
                disabled={backupBusy}
                onClick={() => void handleCreateBackup()}
              >
                {backupBusy ? t('backup.creating') : t('backup.create')}
              </Button>
              {backupNotice && (
                <p className="text-xs text-primary-600 dark:text-primary-400">{backupNotice}</p>
              )}
              {backups.length === 0 ? (
                <p className="text-xs text-gray-400">{t('backup.empty')}</p>
              ) : (
                <ul className="space-y-1">
                  {backups.map((backup) => (
                    <li key={backup.file} className="flex items-center justify-between gap-2 text-xs">
                      <span className="truncate dark:text-gray-300">
                        {new Date(backup.created_at * 1000).toLocaleString()}
                        <span className="ml-2 text-gray-400">{formatSize(backup.size)}</span>
                      </span>
                      <span className="flex shrink-0 gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          disabled={backupBusy}
                          onClick={() => void handleRestore(backup)}
                        >
                          {t('backup.restore')}
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          disabled={backupBusy}
                          onClick={() => void handleDeleteBackup(backup)}
                        >
                          {t('backup.delete')}
                        </Button>
                      </span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium">{t('update.title')}</label>
            <div className="space-y-2 rounded-md border border-gray-200 p-3 text-sm dark:border-gray-700">
              <p className="text-xs text-gray-500 dark:text-gray-400">
                {t('update.currentVersion', { version: __APP_VERSION__ })}
              </p>
              {updateState.kind === 'idle' && (
                <Button variant="outline" size="sm" onClick={() => void handleCheckUpdate()}>
                  {t('update.check')}
                </Button>
              )}
              {updateState.kind === 'checking' && (
                <p className="text-xs text-gray-500">{t('update.checking')}</p>
              )}
              {updateState.kind === 'up-to-date' && (
                <p className="text-xs text-green-600 dark:text-green-400">{t('update.upToDate')}</p>
              )}
              {updateState.kind === 'available' && (
                <div className="space-y-2">
                  <p className="text-xs text-primary-600 dark:text-primary-400">
                    {t('update.available', { version: updateState.update.version })}
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => void handleInstall(updateState.update)}
                  >
                    {t('update.install')}
                  </Button>
                </div>
              )}
              {updateState.kind === 'downloading' && (
                <p className="text-xs text-gray-500">
                  {t('update.downloading', { percent: updateState.percent })}
                </p>
              )}
              {updateState.kind === 'ready' && (
                <div className="space-y-2">
                  <p className="text-xs text-green-600 dark:text-green-400">{t('update.ready')}</p>
                  <Button variant="outline" size="sm" onClick={() => void restartApp()}>
                    {t('update.restart')}
                  </Button>
                </div>
              )}
              {updateState.kind === 'error' && (
                <p className="text-xs text-red-500">
                  {t('update.error', { message: updateState.message })}
                </p>
              )}
            </div>
          </div>

          <div className="flex justify-end">
            <Button onClick={() => setIsOpen(false)}>{t('settings.done')}</Button>
          </div>
        </div>
      </Dialog>
    </>
  );
}
