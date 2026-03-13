import { useState } from 'react';
import { Download, Upload, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { exportVault, importVault, type ImportResult } from '@/lib/tauri';
import { useVaultStore } from '@/stores/vault';
import { open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';

type ImportMode = 'merge' | 'replace';

export function ImportExportModal() {
  const [isOpen, setIsOpen] = useState(false);
  const [importMode, setImportMode] = useState<ImportMode>('merge');
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const loadGroups = useVaultStore((s) => s.loadGroups);
  const loadItems = useVaultStore((s) => s.loadItems);

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
      setError(e instanceof Error ? e.message : 'Export failed');
    } finally {
      setIsExporting(false);
    }
  };

  const handleImport = async () => {
    setIsImporting(true);
    setError(null);
    setImportResult(null);
    
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
      setError(e instanceof Error ? e.message : 'Import failed');
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
        <Download className="mr-2 h-4 w-4" /> Import/Export
      </Button>

      <Dialog open={isOpen} onClose={handleClose} title="Import/Export Vault">
        <div className="space-y-6">
          {error && (
            <div className="flex items-center gap-2 rounded-md bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
              <AlertCircle className="h-4 w-4" />
              {error}
            </div>
          )}

          {importResult && (
            <div className="rounded-md bg-green-50 p-3 text-sm dark:bg-green-900/20">
              <p className="font-medium text-green-700 dark:text-green-400">Import Complete</p>
              <p className="text-green-600 dark:text-green-500">
                Imported {importResult.groups_imported} groups, {importResult.items_imported} items
              </p>
              {(importResult.groups_skipped > 0 || importResult.items_skipped > 0) && (
                <p className="text-yellow-600 dark:text-yellow-400">
                  Skipped {importResult.groups_skipped} groups, {importResult.items_skipped} items (already exist)
                </p>
              )}
            </div>
          )}

          <div>
            <h3 className="mb-2 font-medium dark:text-gray-100">Export Vault</h3>
            <p className="mb-3 text-sm text-gray-500 dark:text-gray-400">
              Export your vault data to a JSON file. Data remains encrypted.
            </p>
            <Button onClick={handleExport} disabled={isExporting}>
              <Download className="mr-2 h-4 w-4" />
              {isExporting ? 'Exporting...' : 'Export to File'}
            </Button>
          </div>

          <div className="border-t border-gray-200 pt-4 dark:border-gray-700">
            <h3 className="mb-2 font-medium dark:text-gray-100">Import Vault</h3>
            <p className="mb-3 text-sm text-gray-500 dark:text-gray-400">
              Import vault data from a JSON file. Requires same master password.
            </p>
            
            <div className="mb-3">
              <label className="mb-1 block text-sm text-gray-500 dark:text-gray-400">Import Mode</label>
              <select
                value={importMode}
                onChange={(e) => setImportMode(e.target.value as ImportMode)}
                className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
              >
                <option value="merge">Merge (keep existing, add new)</option>
                <option value="replace">Replace (delete existing, import all)</option>
              </select>
            </div>

            <Button variant="outline" onClick={handleImport} disabled={isImporting}>
              <Upload className="mr-2 h-4 w-4" />
              {isImporting ? 'Importing...' : 'Import from File'}
            </Button>
          </div>

          <div className="flex justify-end border-t border-gray-200 pt-4 dark:border-gray-700">
            <Button variant="outline" onClick={handleClose}>
              Close
            </Button>
          </div>
        </div>
      </Dialog>
    </>
  );
}