import { useEffect } from 'react';
import { Plus, Star } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function ItemList() {
  const items = useVaultStore((s) => s.items);
  const searchItems = useVaultStore((s) => s.searchItems);
  const clearSearch = useVaultStore((s) => s.clearSearch);
  const searchResults = useVaultStore((s) => s.searchResults);
  const selectItem = useVaultStore((s) => s.selectItem);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const openAddItemModal = useUIStore((s) => s.openAddItemModal);
  const searchQuery = useUIStore((s) => s.searchQuery);
  const sortBy = useUIStore((s) => s.sortBy);
  const setSortBy = useUIStore((s) => s.setSortBy);
  const showFavoritesOnly = useUIStore((s) => s.showFavoritesOnly);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);

  // 输入搜索词时走后端 search_items（LIKE 匹配 title/subtitle/username），防抖 200ms
  useEffect(() => {
    const q = searchQuery.trim();
    if (q === '') {
      clearSearch();
      return;
    }
    const timer = setTimeout(() => {
      void searchItems(q);
    }, 200);
    return () => clearTimeout(timer);
  }, [searchQuery, searchItems, clearSearch]);

  // 结果带 query 快照：过期响应（慢于后续输入到达）不生效；query 已被清空时回退到已加载列表
  const searchActive =
    searchResults !== null && searchResults.query === searchQuery.trim();
  const baseItems = searchActive ? searchResults.items : items;

  const filteredAndSortedItems = baseItems
    .filter((item) => {
      if (showFavoritesOnly && !item.is_favorite) return false;
      if (selectedGroupId && item.group_id !== selectedGroupId) return false;
      if (searchActive) return true; // 后端已完成 title/subtitle/username 匹配
      if (searchQuery === '') return true;
      return (
        item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.subtitle.toLowerCase().includes(searchQuery.toLowerCase())
      );
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name_asc':
          return a.title.localeCompare(b.title);
        case 'name_desc':
          return b.title.localeCompare(a.title);
        case 'created':
          return b.created_at - a.created_at;
        case 'updated':
          return b.updated_at - a.updated_at;
        default:
          return 0;
      }
    });

  return (
    <div className="flex h-full w-80 flex-col border-r border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800">
      <div className="flex items-center justify-between border-b border-gray-200 p-4 dark:border-gray-700">
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as any)}
          className="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
        >
          <option value="name_asc">Name (A-Z)</option>
          <option value="name_desc">Name (Z-A)</option>
          <option value="created">Created (newest)</option>
          <option value="updated">Modified (newest)</option>
        </select>
        <Button size="sm" onClick={openAddItemModal}>
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {filteredAndSortedItems.length === 0 ? (
          <div className="p-4 text-center text-gray-500 dark:text-gray-400">
            {searchActive
              ? 'No items match your search'
              : showFavoritesOnly
                ? 'No favorite items'
                : 'No items'}
          </div>
        ) : (
          filteredAndSortedItems.map((item) => (
            <button
              key={item.id}
              onClick={() => selectItem(item.id)}
              className={`w-full border-b border-gray-200 p-3 text-left hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-700 ${
                selectedItem?.id === item.id ? 'bg-primary-50 dark:bg-primary-900/30' : ''
              }`}
            >
              <div className="flex items-center gap-2">
                <span className="font-medium dark:text-gray-100">{item.title}</span>
                {item.is_favorite && <Star className="h-3 w-3 fill-yellow-400 text-yellow-400" />}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">{item.subtitle}</div>
            </button>
          ))
        )}
      </div>
    </div>
  );
}