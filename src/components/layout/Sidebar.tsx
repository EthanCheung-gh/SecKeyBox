import { Plus, Lock, MoreVertical, ShieldCheck } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { ThemeToggle } from '@/components/ui/theme-toggle';
import { ImportExportModal } from '@/components/modals/ImportExportModal';
import { ChangePasswordModal } from '@/components/modals/ChangePasswordModal';
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
  const openSecurityPanel = useUIStore((s) => s.openSecurityPanel);
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
    <div className="flex h-full w-60 flex-col border-r border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-800">
      <div className="p-4">
        <input
          type="text"
          placeholder="Search..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
        />
      </div>
      
      <div className="flex-1 overflow-y-auto px-2">
        <button
          onClick={() => handleSelectGroup(null)}
          className={`w-full rounded-md px-3 py-2 text-left text-sm ${
            selectedGroupId === null && !showFavoritesOnly 
              ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-100' 
              : 'hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300'
          }`}
        >
          📁 All Items
        </button>
        
        <button
          onClick={handleSelectFavorites}
          className={`w-full rounded-md px-3 py-2 text-left text-sm ${
            showFavoritesOnly 
              ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-100' 
              : 'hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300'
          }`}
        >
          ⭐ Favorites
        </button>
        
        <div className="my-2 border-t border-gray-200 dark:border-gray-700" />
        
        {groups.map((group) => {
          return (
            <div key={group.id} className="relative group flex items-center">
              <button
                onClick={() => handleSelectGroup(group.id)}
                className={`flex-1 rounded-md px-3 py-2 text-left text-sm ${
                  selectedGroupId === group.id
                    ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-100'
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300'
                }`}
              >
                {group.icon || '📂'} {group.name}
              </button>
              <div className="relative">
                <button
                  onClick={() => setMenuOpenId(menuOpenId === group.id ? null : group.id)}
                  className="rounded p-1 opacity-0 group-hover:opacity-100 hover:bg-gray-200 dark:hover:bg-gray-600 dark:text-gray-400"
                >
                  <MoreVertical className="h-4 w-4" />
                </button>
                {menuOpenId === group.id && (
                  <div className="absolute right-0 top-6 z-10 w-24 rounded-md border border-gray-200 bg-white shadow-lg dark:border-gray-600 dark:bg-gray-700">
                    <button
                      onClick={() => {
                        openEditGroupModal(group.id);
                        setMenuOpenId(null);
                      }}
                      className="block w-full px-3 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-600 dark:text-gray-200"
                    >
                      Rename
                    </button>
                    <button
                      onClick={() => {
                        openDeleteConfirm('group', group.id, group.name);
                        setMenuOpenId(null);
                      }}
                      className="block w-full px-3 py-2 text-left text-sm text-red-500 hover:bg-gray-100 dark:hover:bg-gray-600"
                    >
                      Delete
                    </button>
                  </div>
                )}
              </div>
            </div>
          );
        })}
        
        <Button
          variant="ghost"
          size="sm"
          className="mt-2 w-full justify-start text-gray-500 dark:text-gray-400"
          onClick={openAddGroupModal}
        >
          <Plus className="mr-2 h-4 w-4" /> Add Group
        </Button>
      </div>
      
      <div className="border-t border-gray-200 p-2 dark:border-gray-700">
        <Button
          variant="ghost"
          size="sm"
          className="w-full justify-start text-gray-500 dark:text-gray-400"
          onClick={openSecurityPanel}
        >
          <ShieldCheck className="mr-2 h-4 w-4" /> 安全体检
        </Button>
        <ImportExportModal />
        <ChangePasswordModal />
        <div className="mt-2 flex items-center justify-between">
          <Button
            variant="ghost"
            size="sm"
            className="text-gray-500 dark:text-gray-400"
            onClick={lock}
          >
            <Lock className="mr-2 h-4 w-4" /> Lock
          </Button>
          <ThemeToggle />
        </div>
      </div>
    </div>
  );
}