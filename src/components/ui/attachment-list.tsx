import { useCallback, useEffect, useRef, useState } from 'react';
import { Download, Paperclip, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  addAttachment,
  deleteAttachment,
  listAttachments,
  saveAttachmentToPath,
} from '@/lib/tauri';
import { useI18n } from '@/i18n';
import type { AttachmentMeta } from '@/types';

const MAX_BYTES = 5 * 1024 * 1024;

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = String(reader.result ?? '');
      const comma = result.indexOf(',');
      resolve(comma >= 0 ? result.slice(comma + 1) : result);
    };
    reader.onerror = () => reject(reader.error ?? new Error('Failed to read file'));
    reader.readAsDataURL(file);
  });
}

/**
 * Encrypted attachments for one item (SSH keys, certificates, .env files).
 * Files are read in the webview and handed to the backend, which encrypts
 * them with the vault key; nothing unencrypted touches the database.
 */
export function AttachmentList({ itemId }: { itemId: string }) {
  const { t } = useI18n();
  const [attachments, setAttachments] = useState<AttachmentMeta[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const reload = useCallback(async () => {
    try {
      setAttachments(await listAttachments(itemId));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [itemId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const handleFiles = async (files: FileList | null) => {
    if (!files || files.length === 0) return;
    setBusy(true);
    setError(null);
    try {
      for (const file of Array.from(files)) {
        if (file.size > MAX_BYTES) {
          setError(t('attachments.tooLarge', { name: file.name }));
          continue;
        }
        const data = await readFileAsBase64(file);
        await addAttachment(itemId, file.name, file.type || undefined, data);
      }
      await reload();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
      if (inputRef.current) inputRef.current.value = '';
    }
  };

  const handleSave = async (meta: AttachmentMeta) => {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({ defaultPath: meta.filename });
      if (!path) return;
      await saveAttachmentToPath(itemId, meta.id, path);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async (meta: AttachmentMeta) => {
    if (!window.confirm(t('attachments.deleteConfirm', { name: meta.filename }))) return;
    try {
      await deleteAttachment(itemId, meta.id);
      await reload();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="mt-6 border-t border-gray-200 pt-4 dark:border-gray-700">
      <div className="mb-2 flex items-center justify-between">
        <label className="flex items-center gap-1 text-sm font-medium text-gray-500 dark:text-gray-300">
          <Paperclip className="h-4 w-4" /> {t('attachments.title')}
        </label>
        <Button
          variant="outline"
          size="sm"
          disabled={busy}
          onClick={() => inputRef.current?.click()}
        >
          {busy ? t('attachments.adding') : t('attachments.addFile')}
        </Button>
        <input
          ref={inputRef}
          type="file"
          multiple
          className="hidden"
          onChange={(e) => void handleFiles(e.target.files)}
        />
      </div>

      {error && <p className="mb-2 text-xs text-red-500 dark:text-red-400">{error}</p>}

      {attachments.length === 0 ? (
        <p className="text-xs text-gray-400 dark:text-gray-300">
          {t('attachments.empty')}
        </p>
      ) : (
        <ul className="space-y-1">
          {attachments.map((meta) => (
            <li
              key={meta.id}
              className="flex items-center justify-between rounded-md bg-gray-50 px-2 py-1.5 dark:bg-gray-800"
            >
              <div className="min-w-0">
                <p className="truncate text-sm dark:text-gray-200">{meta.filename}</p>
                <p className="text-xs text-gray-400 dark:text-gray-300">{formatSize(meta.size)}</p>
              </div>
              <div className="flex shrink-0 gap-1">
                <Button variant="ghost" size="sm" onClick={() => void handleSave(meta)}>
                  <Download className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="sm" onClick={() => void handleDelete(meta)}>
                  <Trash2 className="h-4 w-4 text-red-500 dark:text-red-400" />
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
