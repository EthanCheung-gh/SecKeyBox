import { useEffect, useState } from 'react';
import { Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { getSecuritySettings } from '@/lib/tauri';
import { useVaultStore } from '@/stores/vault';
import { useI18n, groupDisplayName } from '@/i18n';

/** Fallback window when the settings call fails; mirrors the Rust default. */
const DEFAULT_RETENTION_DAYS = 30;

export function TrashPanel() {
  const { t } = useI18n();
  const trashItems = useVaultStore((s) => s.trashItems);
  const trashCount = useVaultStore((s) => s.trashCount);
  const error = useVaultStore((s) => s.error);
  const loadTrash = useVaultStore((s) => s.loadTrash);
  const restoreTrashItem = useVaultStore((s) => s.restoreTrashItem);
  const purgeTrashItem = useVaultStore((s) => s.purgeTrashItem);
  const emptyTrash = useVaultStore((s) => s.emptyTrash);

  const [retentionDays, setRetentionDays] = useState(DEFAULT_RETENTION_DAYS);
  const [loading, setLoading] = useState(true);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [emptying, setEmptying] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      setLoading(true);
      try {
        await loadTrash();
      } finally {
        if (!cancelled) setLoading(false);
      }
      // The retention window is display-only here; a failed read keeps the default.
      try {
        const settings = await getSecuritySettings();
        if (!cancelled) setRetentionDays(settings.trash_retention_days);
      } catch {
        // keep DEFAULT_RETENTION_DAYS
      }
    };
    void load();
    return () => {
      cancelled = true;
    };
  }, [loadTrash]);

  const handleRestore = async (id: string) => {
    setBusyId(id);
    try {
      await restoreTrashItem(id);
    } finally {
      setBusyId(null);
    }
  };

  const handlePurge = async (id: string, title: string) => {
    if (!window.confirm(t('trash.purgeConfirm', { name: title }))) return;
    setBusyId(id);
    try {
      await purgeTrashItem(id);
    } finally {
      setBusyId(null);
    }
  };

  const handleEmptyTrash = async () => {
    if (trashCount === 0) return;
    if (!window.confirm(t('trash.emptyConfirm', { count: trashCount }))) return;
    setEmptying(true);
    try {
      await emptyTrash();
    } finally {
      setEmptying(false);
    }
  };

  const busy = emptying || busyId !== null;

  return (
    <div className="flex h-full flex-1 flex-col overflow-hidden bg-gray-50 p-6 dark:bg-gray-900">
      <div className="mb-1 flex items-start justify-between gap-4">
        <div className="flex items-center gap-2">
          <Trash2 className="h-5 w-5 text-gray-500 dark:text-gray-400" />
          <h2 className="text-lg font-semibold dark:text-gray-100">{t('trash.title')}</h2>
          <span className="text-xs text-gray-500 dark:text-gray-400">({trashCount})</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          disabled={trashCount === 0 || busy}
          onClick={() => void handleEmptyTrash()}
        >
          {t('trash.emptyTrash')}
        </Button>
      </div>

      <p className="mb-4 text-xs text-gray-500 dark:text-gray-400">
        {retentionDays === 0 ? t('trash.hintNever') : t('trash.hint', { days: retentionDays })}
      </p>

      {error && (
        <p className="mb-3 rounded-md bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
          {error}
        </p>
      )}

      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <p className="text-sm text-gray-500 dark:text-gray-400">{t('common.loading')}</p>
        ) : trashItems.length === 0 ? (
          <p className="rounded-md border border-gray-200 bg-white p-4 text-sm text-gray-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-400">
            {t('trash.empty')}
          </p>
        ) : (
          <ul className="space-y-2">
            {trashItems.map((item) => (
              <li
                key={item.id}
                className="flex items-center justify-between gap-3 rounded-md border border-gray-200 bg-white p-3 dark:border-gray-700 dark:bg-gray-800"
              >
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium dark:text-gray-100">
                    {item.icon || '📂'} {item.title}
                  </div>
                  <div className="mt-1 flex flex-wrap items-center gap-x-2 text-xs text-gray-500 dark:text-gray-400">
                    <span>{t(`type.${item.type}`)}</span>
                    <span>·</span>
                    <span>
                      {t('trash.deletedAt', {
                        time: new Date(item.deleted_at * 1000).toLocaleString(),
                      })}
                    </span>
                    <span>·</span>
                    <span>
                      {item.original_group_id && item.original_group_name
                        ? t('trash.originalGroup', {
                            name: groupDisplayName(
                              { id: item.original_group_id, name: item.original_group_name },
                              t
                            ),
                          })
                        : t('trash.groupDeleted')}
                    </span>
                  </div>
                </div>
                <div className="flex shrink-0 gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    disabled={busyId === item.id}
                    onClick={() => void handleRestore(item.id)}
                  >
                    {t('trash.restore')}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-red-600 hover:bg-red-50 hover:text-red-700 dark:text-red-400 dark:hover:bg-red-900/30"
                    disabled={busyId === item.id}
                    onClick={() => void handlePurge(item.id, item.title)}
                  >
                    {t('trash.purge')}
                  </Button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
