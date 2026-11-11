import { useCallback, useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { archiveDuplicates, findDuplicates } from '@/lib/tauri';
import { useVaultStore } from '@/stores/vault';
import { useI18n, groupDisplayName } from '@/i18n';
import type { DuplicateGroup, MergeReport } from '@/types';

const messageOf = (e: unknown): string => (e instanceof Error ? e.message : String(e));

/**
 * Duplicate review: every group of active items that share an identity key.
 * The user picks the copy to keep; the backend copies the others' metadata
 * over and moves them to the trash (reversible until the trash is purged).
 */
export function DuplicateGroups() {
  const { t } = useI18n();
  const vaultGroups = useVaultStore((s) => s.groups);

  const [groups, setGroups] = useState<DuplicateGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [archiveError, setArchiveError] = useState<string | null>(null);
  const [lastReport, setLastReport] = useState<MergeReport | null>(null);
  /** group key -> id of the copy to keep (defaults to the oldest member). */
  const [selected, setSelected] = useState<Record<string, string>>({});
  const [busyKey, setBusyKey] = useState<string | null>(null);

  const scan = useCallback(async () => {
    setLoading(true);
    try {
      const found = await findDuplicates();
      setGroups(found);
      setError(null);
      // Default to the oldest member of each group (backend sorts oldest first).
      // Keep an explicit choice when that member still exists in the group.
      setSelected((prev) => {
        const next: Record<string, string> = {};
        for (const group of found) {
          const previous = prev[group.key];
          next[group.key] =
            previous && group.items.some((item) => item.id === previous)
              ? previous
              : (group.items[0]?.id ?? '');
        }
        return next;
      });
    } catch (e) {
      setGroups([]);
      setError(messageOf(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void scan();
  }, [scan]);

  const handleArchive = async (group: DuplicateGroup) => {
    const keepId = selected[group.key] ?? group.items[0]?.id;
    const kept = group.items.find((item) => item.id === keepId) ?? group.items[0];
    if (!kept) return;
    const duplicateIds = group.items
      .filter((item) => item.id !== kept.id)
      .map((item) => item.id);
    if (duplicateIds.length === 0) return;
    if (
      !window.confirm(
        t('duplicates.confirm', { title: kept.title, count: duplicateIds.length })
      )
    ) {
      return;
    }

    setBusyKey(group.key);
    setArchiveError(null);
    setLastReport(null);
    try {
      setLastReport(await archiveDuplicates(kept.id, duplicateIds));
      await scan();
      // The merged items leave the item list and land in the trash.
      await useVaultStore.getState().loadItems();
      await useVaultStore.getState().loadTrashCount();
    } catch (e) {
      setArchiveError(messageOf(e));
    } finally {
      setBusyKey(null);
    }
  };

  const groupName = (groupId: string) => {
    const group = vaultGroups.find((g) => g.id === groupId);
    return group ? groupDisplayName(group, t) : groupId;
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <h3 className="text-sm font-medium dark:text-gray-100">{t('duplicates.title')}</h3>
      <p className="mb-3 text-xs text-gray-500 dark:text-gray-300">{t('duplicates.hint')}</p>

      {error && (
        <p className="mb-3 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600 dark:border-red-900 dark:bg-red-900/20 dark:text-red-400">
          {t('duplicates.error', { message: error })}
        </p>
      )}
      {archiveError && (
        <p className="mb-3 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600 dark:border-red-900 dark:bg-red-900/20 dark:text-red-400">
          {t('duplicates.archiveError', { message: archiveError })}
        </p>
      )}
      {lastReport && (
        <p className="mb-3 rounded-md border border-green-200 bg-green-50 p-3 text-sm text-green-700 dark:border-green-900 dark:bg-green-900/20 dark:text-green-400">
          {t('duplicates.result', {
            trashed: lastReport.trashed,
            tags: lastReport.moved_tags,
            attachments: lastReport.moved_attachments,
            fields: lastReport.moved_custom_fields,
            history: lastReport.moved_history,
          })}
        </p>
      )}

      {loading && groups.length === 0 && (
        <p className="text-sm text-gray-500 dark:text-gray-300">{t('common.loading')}</p>
      )}

      {!loading && groups.length === 0 && !error && (
        <p className="rounded-md border border-gray-200 bg-white p-4 text-sm text-gray-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300">
          {t('duplicates.empty')}
        </p>
      )}

      <div className="flex-1 space-y-3 overflow-y-auto">
        {groups.map((group) => {
          const busy = busyKey === group.key;
          const blocked = busyKey !== null;
          return (
            <div
              key={group.key}
              className="rounded-md border border-gray-200 bg-white p-3 dark:border-gray-700 dark:bg-gray-800"
            >
              <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
                <h4 className="text-sm font-medium dark:text-gray-100">
                  {t(`type.${group.type}`)}
                  <span className="ml-1 text-xs text-gray-500 dark:text-gray-300">
                    {t('duplicates.count', { count: group.items.length })}
                  </span>
                </h4>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={blocked || group.items.length < 2}
                  onClick={() => void handleArchive(group)}
                >
                  {busy ? t('duplicates.archiving') : t('duplicates.archive')}
                </Button>
              </div>
              <ul className="space-y-1">
                {group.items.map((item) => (
                  <li key={item.id} className="flex flex-wrap items-center gap-2 text-xs">
                    <label className="flex items-center gap-1">
                      <input
                        type="radio"
                        name={`keep-${group.key}`}
                        className="accent-primary-600"
                        checked={(selected[group.key] ?? group.items[0]?.id) === item.id}
                        disabled={blocked}
                        onChange={() =>
                          setSelected((prev) => ({ ...prev, [group.key]: item.id }))
                        }
                      />
                      <span className="text-gray-500 dark:text-gray-300">
                        {t('duplicates.keep')}
                      </span>
                    </label>
                    <span className="font-medium dark:text-gray-100">{item.title}</span>
                    <span className="text-gray-500 dark:text-gray-300">
                      {groupName(item.group_id)}
                    </span>
                    <span className="text-gray-400 dark:text-gray-300">
                      {t('duplicates.created', {
                        time: new Date(item.created_at * 1000).toLocaleString(),
                      })}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          );
        })}
      </div>
    </div>
  );
}
