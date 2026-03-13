import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { ItemSummary, ItemDetail } from '@/types';

export function QuickSearchWindow() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<ItemSummary[]>([]);
  const [selectedItem, setSelectedItem] = useState<ItemDetail | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWindow().hide();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  useEffect(() => {
    const search = async () => {
      try {
        const items = await invoke<ItemSummary[]>('search_items', { query });
        setResults(items);
      } catch (e) {
        console.error('Search failed:', e);
      }
    };

    const debounce = setTimeout(search, 100);
    return () => clearTimeout(debounce);
  }, [query]);

  const handleCopy = async (text: string) => {
    await invoke('copy_to_clipboard', { text });
    setTimeout(() => {
      invoke('clear_clipboard');
    }, 30000);
    getCurrentWindow().hide();
  };

  const handleSelectItem = async (id: string) => {
    try {
      const detail = await invoke<ItemDetail>('get_item_detail', { id });
      setSelectedItem(detail);
    } catch (e) {
      console.error('Failed to get item detail:', e);
    }
  };

  const renderCopyButtons = () => {
    if (!selectedItem) return null;

    switch (selectedItem.type) {
      case 'account':
        return (
          <div className="space-y-1">
            <button
              onClick={() => handleCopy(selectedItem.username)}
              className="w-full rounded bg-gray-100 px-3 py-2 text-left text-sm hover:bg-gray-200"
            >
              Copy username: {selectedItem.username}
            </button>
            <button
              onClick={() => handleCopy(selectedItem.password)}
              className="w-full rounded bg-gray-100 px-3 py-2 text-left text-sm hover:bg-gray-200"
            >
              Copy password: ••••••••
            </button>
          </div>
        );
      case 'api_key':
        return (
          <div className="space-y-1">
            <button
              onClick={() => handleCopy(selectedItem.key_value)}
              className="w-full rounded bg-gray-100 px-3 py-2 text-left text-sm hover:bg-gray-200"
            >
              Copy API key: ••••{selectedItem.key_value.slice(-4)}
            </button>
          </div>
        );
      case 'env_var':
        return (
          <div className="space-y-1">
            {selectedItem.variables.map((v, i) => (
              <button
                key={i}
                onClick={() => handleCopy(v.value)}
                className="w-full rounded bg-gray-100 px-3 py-2 text-left text-sm hover:bg-gray-200"
              >
                Copy {v.key}: ••••••••
              </button>
            ))}
          </div>
        );
    }
  };

  return (
    <div className="h-full bg-white p-3">
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search items..."
        className="w-full rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
      />

      {selectedItem ? (
        <div className="mt-2">
          <button
            onClick={() => setSelectedItem(null)}
            className="mb-2 text-xs text-gray-500 hover:text-gray-700"
          >
            ← Back to results
          </button>
          <div className="text-sm font-medium text-gray-700">{selectedItem.title}</div>
          {renderCopyButtons()}
        </div>
      ) : (
        <div className="mt-2 max-h-[200px] overflow-y-auto">
          {results.length === 0 ? (
            <p className="py-4 text-center text-sm text-gray-400">No items found</p>
          ) : (
            results.slice(0, 10).map((item) => (
              <button
                key={item.id}
                onClick={() => handleSelectItem(item.id)}
                className="w-full rounded px-2 py-2 text-left hover:bg-gray-100"
              >
                <div className="text-sm font-medium">{item.title}</div>
                <div className="text-xs text-gray-500">{item.subtitle}</div>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}