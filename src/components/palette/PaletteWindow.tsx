import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { ItemSummary, ItemDetail } from '@/types';

// 全局快捷键呼出的迷你搜索条：搜索 → ↑/↓ 选择 → 回车复制主密钥 → Esc 关闭。
// 锁定时后端返回空列表；这里按空结果渲染即可（调用前不会解锁）。
export function PaletteWindow() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<ItemSummary[]>([]);
  const [selected, setSelected] = useState(0);
  const [status, setStatus] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(async () => {
      try {
        const items = await invoke<ItemSummary[]>('search_items', { query });
        if (!cancelled) {
          setResults(items);
          setSelected(0);
          setStatus(null);
        }
      } catch (e) {
        if (!cancelled) setStatus(`搜索失败: ${String(e)}`);
      }
    }, 200);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [query]);

  const copySelected = async () => {
    const item = results[selected];
    if (!item) return;
    try {
      const detail = await invoke<ItemDetail>('get_item_detail', { id: item.id });
      const value =
        detail.type === 'account'
          ? detail.password
          : detail.type === 'api_key'
            ? detail.key_value
            : detail.type === 'env_var'
              ? detail.variables[0]?.value ?? ''
              : detail.type === 'database'
                ? detail.password ?? ''
                : detail.type === 'ssh'
                  ? detail.password ?? ''
                  : detail.type === 'cloud'
                    ? detail.secret
                    : detail.type === 'license'
                      ? detail.license_key
                      : detail.password;
      if (value) {
        await invoke('copy_to_clipboard', { text: value });
        setStatus(`已复制 ${item.title} 的密钥`);
      } else {
        setStatus(`${item.title} 没有可复制的密钥`);
      }
    } catch (e) {
      setStatus(`复制失败: ${String(e)}`);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      void getCurrentWindow().hide();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelected((s) => Math.min(s + 1, results.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelected((s) => Math.max(s - 1, 0));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      void copySelected();
    }
  };

  const isBrowserMock = '__SECKEYBOX_MOCK__' in window;

  return (
    <div className="flex h-screen flex-col bg-white p-3 dark:bg-gray-900">
      <div className="mb-2 flex items-center gap-2">
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Search and press Enter to copy..."
          className="min-w-0 flex-1 rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:border-blue-500 focus:outline-none dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
        />
        <button
          onClick={() => {
            if (isBrowserMock) {
              window.location.assign('/');
            } else {
              void getCurrentWindow().hide();
              void invoke('plugin:window|show', { label: 'main' });
              void invoke('plugin:window|set_focus', { label: 'main' });
            }
          }}
          className="shrink-0 rounded-md border border-gray-300 px-2.5 py-2 text-xs text-gray-600 hover:bg-gray-100 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700"
          title="打开主窗口"
        >
          ⌂
        </button>
      </div>

      {status && <p className="mb-1 text-xs text-gray-500 dark:text-gray-400">{status}</p>}

      <div className="max-h-[280px] flex-1 overflow-y-auto">
        {results.length === 0 ? (
          <p className="py-4 text-center text-sm text-gray-400">
            {query ? 'No items match' : 'Type to search the vault'}
          </p>
        ) : (
          results.slice(0, 10).map((item, i) => (
            <button
              key={item.id}
              onClick={() => setSelected(i)}
              onDoubleClick={() => void copySelected()}
              className={`w-full rounded-md px-2 py-2 text-left ${
                i === selected ? 'bg-primary-100 dark:bg-primary-900' : 'hover:bg-gray-100 dark:hover:bg-gray-700'
              }`}
            >
              <div className="text-sm font-medium dark:text-gray-100">{item.title}</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">{item.subtitle}</div>
            </button>
          ))
        )}
      </div>

      <p className="mt-1 text-center text-[10px] text-gray-400">
        ↑↓ 选择 · Enter 复制主密钥 · Esc 关闭
      </p>
    </div>
  );
}
