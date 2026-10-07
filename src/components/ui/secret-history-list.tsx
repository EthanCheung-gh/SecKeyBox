import { useState } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';
import type { SecretHistoryEntry } from '@/types';

const formatTime = (ts: number) => new Date(ts * 1000).toLocaleString();

// 详情面板里的历史密文折叠列表：轮换密码/密钥前的旧值（带时间戳，可复制）。
export function SecretHistoryList({ entries }: { entries: SecretHistoryEntry[] }) {
  const [open, setOpen] = useState(false);
  if (entries.length === 0) return null;

  return (
    <div className="border-t border-gray-200 pt-3 dark:border-gray-700">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
      >
        {open ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
        历史密钥（{entries.length}）
      </button>
      {open && (
        <div className="mt-2 space-y-1">
          {entries.map((entry, i) => (
            <div key={i} className="flex items-center justify-between text-xs dark:text-gray-400">
              <span className="text-gray-400">{formatTime(entry.changed_at)}</span>
              <button
                onClick={() => navigator.clipboard.writeText(entry.value).catch(() => {})}
                className="font-mono text-gray-500 hover:text-gray-800 dark:hover:text-gray-200"
                title="复制此历史值"
              >
                {entry.value.slice(0, 12)}
                {entry.value.length > 12 ? '…' : ''}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
