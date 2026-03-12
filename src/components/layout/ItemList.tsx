import { Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function ItemList() {
  const items = useVaultStore((s) => s.items);
  const selectItem = useVaultStore((s) => s.selectItem);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const openAddItemModal = useUIStore((s) => s.openAddItemModal);
  const searchQuery = useUIStore((s) => s.searchQuery);
  const sortBy = useUIStore((s) => s.sortBy);
  const setSortBy = useUIStore((s) => s.setSortBy);

  const filteredAndSortedItems = items
    .filter((item) =>
      searchQuery === '' ||
      item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.subtitle.toLowerCase().includes(searchQuery.toLowerCase())
    )
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
    <div className="flex h-full w-80 flex-col border-r bg-white">
      <div className="flex items-center justify-between border-b p-4">
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as any)}
          className="rounded-md border border-gray-300 px-2 py-1 text-sm"
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
          <div className="p-4 text-center text-gray-500">No items</div>
        ) : (
          filteredAndSortedItems.map((item) => (
            <button
              key={item.id}
              onClick={() => selectItem(item.id)}
              className={`w-full border-b p-3 text-left hover:bg-gray-50 ${
                selectedItem?.id === item.id ? 'bg-primary-50' : ''
              }`}
            >
              <div className="font-medium">{item.title}</div>
              <div className="text-sm text-gray-500">{item.subtitle}</div>
            </button>
          ))
        )}
      </div>
    </div>
  );
}