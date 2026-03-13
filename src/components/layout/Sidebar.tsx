import { Plus, Lock, MoreVertical } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function Sidebar() {
  const groups = useVaultStore((s) => s.groups);
  const loadItems = useVaultStore((s) => s.loadItems);
  const lock = useVaultStore((s) => s.lock);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);
  const showFavoritesOnly = useUIStore((s) => s.showFavoritesOnly);
  const searchQuery = useUIStore((s) => s.searchQuery);
  const setSearchQuery = useUIStore((s) => s.setSearchQuery);
  const setSelectedGroup = useUIStore((s) => s.setSelectedGroup);
  const setShowFavoritesOnly = useUIStore((s) => s.setShowFavoritesOnly);
  const openAddGroupModal = useUIStore((s) => s.openAddGroupModal);
  const openEditGroupModal = useUIStore((s) => s.openEditGroupModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);

  const handleSelectGroup = (id: string | null) => {
    setSelectedGroup(id);
    loadItems(id || undefined);
  };

  const handleSelectFavorites = () => {
    setShowFavoritesOnly(true);
    loadItems(undefined);
  };

  return (
    <div className="flex h-full w-60 flex-col border-r bg-gray-50">
      <div className="p-4">
        <input
          type="text"
          placeholder="Search..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
        />
      </div>
      
      <div className="flex-1 overflow-y-auto px-2">
        <button
          onClick={() => handleSelectGroup(null)}
          className={`w-full rounded-md px-3 py-2 text-left text-sm ${
            selectedGroupId === null && !showFavoritesOnly ? 'bg-primary-100 text-primary-700' : 'hover:bg-gray-100'
          }`}
        >
          📁 All Items
        </button>
        
        <button
          onClick={handleSelectFavorites}
          className={`w-full rounded-md px-3 py-2 text-left text-sm ${
            showFavoritesOnly ? 'bg-primary-100 text-primary-700' : 'hover:bg-gray-100'
          }`}
        >
          ⭐ Favorites
        </button>
        
        <div className="my-2 border-t" />
        
        {groups.map((group) => {
          return (
            <div key={group.id} className="relative group flex items-center">
              <button
                onClick={() => handleSelectGroup(group.id)}
                className={`flex-1 rounded-md px-3 py-2 text-left text-sm ${
                  selectedGroupId === group.id ? 'bg-primary-100 text-primary-700' : 'hover:bg-gray-100'
                }`}
              >
                {group.icon || '📂'} {group.name}
              </button>
              {!group.id.startsWith('built-in-') && (
                <div className="relative">
                  <button
                    onClick={() => setMenuOpenId(menuOpenId === group.id ? null : group.id)}
                    className="rounded p-1 opacity-0 group-hover:opacity-100 hover:bg-gray-200"
                  >
                    <MoreVertical className="h-4 w-4" />
                  </button>
                  {menuOpenId === group.id && (
                    <div className="absolute right-0 top-6 z-10 w-24 rounded-md border bg-white shadow-lg">
                      <button
                        onClick={() => {
                          openEditGroupModal(group.id);
                          setMenuOpenId(null);
                        }}
                        className="block w-full px-3 py-2 text-left text-sm hover:bg-gray-100"
                      >
                        Rename
                      </button>
                      <button
                        onClick={() => {
                          openDeleteConfirm('group', group.id, group.name);
                          setMenuOpenId(null);
                        }}
                        className="block w-full px-3 py-2 text-left text-sm text-red-500 hover:bg-gray-100"
                      >
                        Delete
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>
          );
        })}
        
        <Button
          variant="ghost"
          size="sm"
          className="mt-2 w-full justify-start text-gray-500"
          onClick={openAddGroupModal}
        >
          <Plus className="mr-2 h-4 w-4" /> Add Group
        </Button>
      </div>
      
      <div className="border-t p-2">
        <Button
          variant="ghost"
          size="sm"
          className="w-full justify-start text-gray-500"
          onClick={lock}
        >
          <Lock className="mr-2 h-4 w-4" /> Lock
        </Button>
      </div>
    </div>
  );
}