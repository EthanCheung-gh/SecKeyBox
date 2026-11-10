import type { IntegrityReport } from '@/types';
import { useI18n } from '@/i18n';

/**
 * Renders one integrity report. The backend never sends plaintext or
 * ciphertext, so showing every field of the report is safe.
 */
export function IntegrityReportView({ report }: { report: IntegrityReport }) {
  const { t } = useI18n();

  const healthy =
    report.failures.length === 0 &&
    report.sqlite_integrity === 'ok' &&
    report.foreign_key_violations.length === 0;

  return (
    <div className="space-y-2 rounded-md border border-gray-200 p-3 text-xs dark:border-gray-700">
      {healthy && (
        <div className="rounded-md border border-green-200 bg-green-50 p-2 dark:border-green-900 dark:bg-green-900/20">
          <p className="text-green-700 dark:text-green-400">
            {t('integrity.ok', { count: report.checked_fields })}
          </p>
          <p className="mt-1 text-gray-500 dark:text-gray-400">
            {t('integrity.duration', { ms: report.duration_ms })}
          </p>
        </div>
      )}

      {!healthy && report.failures.length > 0 && (
        <p className="rounded-md border border-red-200 bg-red-50 p-2 text-red-600 dark:border-red-900 dark:bg-red-900/20 dark:text-red-400">
          {t('integrity.failed', { count: report.failures.length })}
        </p>
      )}

      {!healthy && report.failures.length === 0 && (
        <p className="rounded-md border border-yellow-200 bg-yellow-50 p-2 text-yellow-700 dark:border-yellow-900 dark:bg-yellow-900/20 dark:text-yellow-400">
          {t('integrity.structureIssues')}
        </p>
      )}

      <ul className="space-y-1 text-gray-500 dark:text-gray-400">
        <li>{t('integrity.checkedFields', { count: report.checked_fields })}</li>
        <li>{t('integrity.sqlite', { status: report.sqlite_integrity })}</li>
        <li>
          {t('integrity.foreignKeys', { count: report.foreign_key_violations.length })}
          {report.foreign_key_violations.length > 0 && (
            <ul className="mt-1 list-inside list-disc font-mono">
              {report.foreign_key_violations.map((violation, index) => (
                <li key={index}>{violation}</li>
              ))}
            </ul>
          )}
        </li>
      </ul>

      {report.failures.length > 0 && (
        <ul className="space-y-2 border-t border-gray-200 pt-2 dark:border-gray-700">
          {report.failures.map((failure, index) => {
            const typeKey = `type.${failure.item_type}`;
            // `t` falls back to the key itself, so an unknown type renders no label.
            const typeLabel = t(typeKey);
            return (
              <li key={`${failure.item_id}-${failure.scope}-${failure.field}-${index}`}>
                <div className="flex flex-wrap items-center gap-1 dark:text-gray-200">
                  <span className="font-medium">{failure.item_title || failure.item_id}</span>
                  {typeLabel !== typeKey && (
                    <span className="text-gray-400">· {typeLabel}</span>
                  )}
                </div>
                <p className="text-gray-500 dark:text-gray-400">
                  {failure.scope} · {failure.field}
                </p>
                <p className="break-words font-mono text-gray-400 dark:text-gray-500">
                  {failure.reason}
                </p>
              </li>
            );
          })}
        </ul>
      )}

      {!healthy && (
        <p className="text-gray-500 dark:text-gray-400">{t('integrity.restoreHint')}</p>
      )}
    </div>
  );
}
