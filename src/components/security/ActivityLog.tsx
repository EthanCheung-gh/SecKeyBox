import { useCallback, useEffect, useState } from 'react';
import { ShieldCheck, Activity } from 'lucide-react';
import { getActivityLog } from '@/lib/tauri';
import { useI18n } from '@/i18n';
import type { AuditEntry } from '@/types';

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleString();
}

/**
 * Local activity log: unlock/lock, copies, item and attachment changes,
 * imports/exports. Titles and field names only — never secret values.
 */
export function ActivityLog() {
  const { t } = useI18n();
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      setEntries(await getActivityLog(200));
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const label = (event: string) => {
    const key = `audit.event.${event}`;
    const translated = t(key);
    return translated === key ? event : translated;
  };

  const tone = (event: string) => {
    if (event === 'item.delete' || event === 'attachment.delete') return 'text-red-600 dark:text-red-400';
    if (event.startsWith('vault.lock')) return 'text-yellow-600 dark:text-yellow-400';
    if (event === 'vault.unlock' || event === 'item.create') return 'text-green-600 dark:text-green-400';
    return 'text-gray-700 dark:text-gray-300';
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
          <Activity className="h-4 w-4" />
          {t('audit.hint')}
        </div>
        <button
          onClick={() => void load()}
          className="rounded-md border border-gray-300 px-2 py-1 text-xs text-gray-600 hover:bg-gray-100 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700"
        >
          {t('common.refresh')}
        </button>
      </div>

      {loading && <p className="text-sm text-gray-500">{t('common.loading')}</p>}
      {error && <p className="text-sm text-red-500">{error}</p>}

      {!loading && entries.length === 0 && (
        <p className="rounded-md border border-gray-200 p-4 text-sm text-gray-500 dark:border-gray-700 dark:text-gray-400">
          <ShieldCheck className="mb-1 inline h-4 w-4" /> {t('audit.empty')}
        </p>
      )}

      {entries.length > 0 && (
        <div className="overflow-hidden rounded-md border border-gray-200 dark:border-gray-700">
          <table className="w-full text-sm">
            <thead className="bg-gray-100 text-left text-xs text-gray-500 dark:bg-gray-800 dark:text-gray-400">
              <tr>
                <th className="px-3 py-2 font-medium">{t('audit.colTime')}</th>
                <th className="px-3 py-2 font-medium">{t('audit.colEvent')}</th>
                <th className="px-3 py-2 font-medium">{t('audit.colItem')}</th>
                <th className="px-3 py-2 font-medium">{t('audit.colDetail')}</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => (
                <tr key={entry.id} className="border-t border-gray-100 dark:border-gray-800">
                  <td className="whitespace-nowrap px-3 py-1.5 text-xs text-gray-500 dark:text-gray-400">
                    {formatTime(entry.created_at)}
                  </td>
                  <td className={`whitespace-nowrap px-3 py-1.5 text-xs font-medium ${tone(entry.event)}`}>
                    {label(entry.event)}
                  </td>
                  <td className="max-w-[14rem] truncate px-3 py-1.5 text-xs dark:text-gray-300">
                    {entry.item_title ?? '—'}
                  </td>
                  <td className="max-w-[16rem] truncate px-3 py-1.5 text-xs text-gray-500 dark:text-gray-400">
                    {entry.detail ?? '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
