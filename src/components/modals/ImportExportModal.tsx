import { useState } from 'react';
import { Download, Upload, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { exportVault, importVault, importCsv, importEnv, type ImportResult } from '@/lib/tauri';
import type { ImportToolsResult } from '@/types';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';
import { useI18n } from '@/i18n';

type ImportMode = 'merge' | 'replace';
type ImportTab = 'json' | 'csv' | 'env';

export function ImportExportModal() {
  const { t } = useI18n();
  const [isOpen, setIsOpen] = useState(false);
  const [importMode, setImportMode] = useState<ImportMode>('merge');
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [toolsResult, setToolsResult] = useState<ImportToolsResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [importTab, setImportTab] = useState<ImportTab>('json');
  const [envTitle, setEnvTitle] = useState('');
  const loadGroups = useVaultStore((s) => s.loadGroups);
  const loadItems = useVaultStore((s) => s.loadItems);
  const groups = useVaultStore((s) => s.groups);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);
  const targetGroupId = selectedGroupId || groups[0]?.id || '';

  const handleExport = async () => {
    setIsExporting(true);
    setError(null);
    try {
      const data = await exportVault();
      const filePath = await open({
        defaultPath: `seckeybox-export-${new Date().toISOString().split('T')[0]}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });
      
      if (filePath) {
        await writeTextFile(filePath as string, data);
        setIsOpen(false);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : t('importExport.exportFailed'));
    } finally {
      setIsExporting(false);
    }
  };

  const handleImport = async () => {
    setIsImporting(true);
    setError(null);
    setImportResult(null);
    setToolsResult(null);

    try {
      const filePath = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      });

      if (!filePath) {
        setIsImporting(false);
        return;
      }

      const data = await readTextFile(filePath as string);
      const result = await importVault(data, importMode);
      setImportResult(result);

      await loadGroups();
      await loadItems();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('importExport.importFailed'));
    } finally {
      setIsImporting(false);
    }
  };

  const handleCsvImport = async () => {
    setIsImporting(true);
    setError(null);
    setImportResult(null);
    setToolsResult(null);

    try {
      const filePath = await open({
        filters: [{ name: 'CSV', extensions: ['csv'] }],
        multiple: false,
      });
      if (!filePath) {
        setIsImporting(false);
        return;
      }
      const data = await readTextFile(filePath as string);
      if (!targetGroupId) {
        setError(t('importExport.noGroups'));
        return;
      }
      const result = await importCsv(data, targetGroupId);
      setToolsResult(result);
      await loadGroups();
      await loadItems();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('importExport.csvFailed'));
    } finally {
      setIsImporting(false);
    }
  };

  const handleEnvImport = async () => {
    setIsImporting(true);
    setError(null);
    setImportResult(null);
    setToolsResult(null);

    try {
      const filePath = await open({
        filters: [{ name: 'Env', extensions: ['env', 'txt'] }],
        multiple: false,
      });
      if (!filePath) {
        setIsImporting(false);
        return;
      }
      const content = await readTextFile(filePath as string);
      if (!targetGroupId) {
        setError(t('importExport.noGroups'));
        return;
      }
      const result = await importEnv(
        targetGroupId,
        envTitle.trim() || (filePath as string).split('/').pop() || t('importExport.envFile'),
        content,
      );
      setToolsResult(result);
      await loadGroups();
      await loadItems();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('importExport.envFailed'));
    } finally {
      setIsImporting(false);
    }
  };

  const handleClose = () => {
    setIsOpen(false);
    setImportResult(null);
    setError(null);
  };

  return (
    <>
      <Button
        variant="ghost"
        size="sm"
        className="w-full justify-start text-gray-500 dark:text-gray-400"
        onClick={() => setIsOpen(true)}
      >
        <Download className="mr-2 h-4 w-4" /> {t('importExport.short')}
      </Button>

      <Dialog open={isOpen} onClose={handleClose} title={t('importExport.title')}>
        <div className="space-y-6">
          {error && (
            <div className="flex items-center gap-2 rounded-md bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
              <AlertCircle className="h-4 w-4" />
              {error}
            </div>
          )}

          {importResult && (
            <div className="rounded-md bg-green-50 p-3 text-sm dark:bg-green-900/20">
              <p className="font-medium text-green-700 dark:text-green-400">{t('importExport.importComplete')}</p>
              <p className="text-green-600 dark:text-green-500">
                {t('importExport.importedGroupsItems', { groups: importResult.groups_imported, items: importResult.items_imported })}
              </p>
              {(importResult.groups_skipped > 0 || importResult.items_skipped > 0) && (
                <p className="text-yellow-600 dark:text-yellow-400">
                  {t('importExport.skippedGroupsItems', { groups: importResult.groups_skipped, items: importResult.items_skipped })}
                </p>
              )}
            </div>
          )}

          {toolsResult && (
            <div className="rounded-md bg-green-50 p-3 text-sm dark:bg-green-900/20">
              <p className="font-medium text-green-700 dark:text-green-400">{t('importExport.importComplete')}</p>
              <p className="text-green-600 dark:text-green-500">
                {toolsResult.groups_imported > 0
                  ? t('importExport.importedWithGroups', { items: toolsResult.items_imported, groups: toolsResult.groups_imported })
                  : t('importExport.importedItemsOnly', { items: toolsResult.items_imported })}
                {toolsResult.items_skipped > 0 && t('importExport.skippedRows', { count: toolsResult.items_skipped })}
              </p>
            </div>
          )}

          <div>
            <h3 className="mb-2 font-medium dark:text-gray-100">{t('importExport.exportVault')}</h3>
            <p className="mb-3 text-sm text-gray-500 dark:text-gray-400">
              {t('importExport.exportHint')}
            </p>
            <Button onClick={handleExport} disabled={isExporting}>
              <Download className="mr-2 h-4 w-4" />
              {isExporting ? t('importExport.exporting') : t('importExport.exportToFile')}
            </Button>
          </div>

          <div className="border-t border-gray-200 pt-4 dark:border-gray-700">
            <h3 className="mb-2 font-medium dark:text-gray-100">{t('importExport.importVault')}</h3>
            <div className="mb-3 flex gap-2 text-sm">
              {(
                [
                  ['json', t('importExport.jsonBackup')],
                  ['csv', t('importExport.otherManagers')],
                  ['env', t('importExport.envFile')],
                ] as const
              ).map(([tab, label]) => (
                <button
                  key={tab}
                  onClick={() => setImportTab(tab)}
                  className={`rounded-md border px-2.5 py-1 ${
                    importTab === tab
                      ? 'border-primary-500 text-primary-600 dark:text-primary-400'
                      : 'border-gray-300 text-gray-500 dark:border-gray-600 dark:text-gray-400'
                  }`}
                >
                  {label}
                </button>
              ))}
            </div>

            {importTab === 'json' && (
              <>
                <p className="mb-3 text-sm text-gray-500 dark:text-gray-400">
                  {t('importExport.importHint')}
                </p>
                <div className="mb-3">
                  <label className="mb-1 block text-sm text-gray-500 dark:text-gray-400">{t('importExport.importMode')}</label>
                  <select
                    value={importMode}
                    onChange={(e) => setImportMode(e.target.value as ImportMode)}
                    className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
                  >
                    <option value="merge">{t('importExport.merge')}</option>
                    <option value="replace">{t('importExport.replace')}</option>
                  </select>
                </div>
                <Button variant="outline" onClick={handleImport} disabled={isImporting}>
                  <Upload className="mr-2 h-4 w-4" />
                  {isImporting ? t('importExport.importing') : t('importExport.importFromFile')}
                </Button>
              </>
            )}

            {importTab === 'csv' && (
              <>
                <p className="mb-3 text-sm text-gray-500 dark:text-gray-400">
                  {t('importExport.csvHint', {
                    group: groups.find((g) => g.id === targetGroupId)?.name ?? t('importExport.unselected'),
                  })}
                </p>
                <Button variant="outline" onClick={handleCsvImport} disabled={isImporting}>
                  <Upload className="mr-2 h-4 w-4" />
                  {isImporting ? t('importExport.importing') : t('importExport.selectCsv')}
                </Button>
              </>
            )}

            {importTab === 'env' && (
              <>
                <p className="mb-3 text-sm text-gray-500 dark:text-gray-400">
                  {t('importExport.envHint')} <code className="font-mono">KEY=VALUE</code>
                </p>
                <div className="mb-3">
                  <label className="mb-1 block text-sm text-gray-500 dark:text-gray-400">{t('importExport.envTitle')}</label>
                  <input
                    value={envTitle}
                    onChange={(e) => setEnvTitle(e.target.value)}
                    placeholder={t('importExport.envTitlePlaceholder')}
                    className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
                  />
                </div>
                <Button variant="outline" onClick={handleEnvImport} disabled={isImporting}>
                  <Upload className="mr-2 h-4 w-4" />
                  {isImporting ? t('importExport.importing') : t('importExport.selectEnv')}
                </Button>
              </>
            )}
          </div>

          <div className="flex justify-end border-t border-gray-200 pt-4 dark:border-gray-700">
            <Button variant="outline" onClick={handleClose}>
              {t('common.close')}
            </Button>
          </div>
        </div>
      </Dialog>
    </>
  );
}