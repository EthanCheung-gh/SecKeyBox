import { Plus, Lock, MoreVertical, ShieldCheck, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { ThemeToggle } from '@/components/ui/theme-toggle';
import { ImportExportModal } from '@/components/modals/ImportExportModal';
import { ChangePasswordModal } from '@/components/modals/ChangePasswordModal';
import { SettingsModal } from '@/components/modals/SettingsModal';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { useI18n, groupDisplayName } from '@/i18n';

export function Sidebar() {
  const { t } = useI18n();
  const groups = useVaultStore((s) => s.groups);
  const tags = useVaultStore((s) => s.tags);
  const loadItemsByTag = useVaultStore((s) => s.loadItemsByTag);
  const loadItems = useVaultStore((s) => s.loadItems);
  const lock = useVaultStore((s) => s.lock);
  const trashCount = useVaultStore((s) => s.trashCount);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);
  const selectedTagId = useUIStore((s) => s.selectedTagId);
  const setSelectedTag = useUIStore((s) => s.setSelectedTag);
  const openAddTagModal = useUIStore((s) => s.openAddTagModal);
  const showFavoritesOnly = useUIStore((s) => s.showFavoritesOnly);
  const searchQuery = useUIStore((s) => s.searchQuery);
  const setSearchQuery = useUIStore((s) => s.setSearchQuery);
  const setSelectedGroup = useUIStore((s) => s.setSelectedGroup);
  const setShowFavoritesOnly = useUIStore((s) => s.setShowFavoritesOnly);
  const openAddGroupModal = useUIStore((s) => s.openAddGroupModal);
  const openEditGroupModal = useUIStore((s) => s.openEditGroupModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const openSecurityPanel = useUIStore((s) => s.openSecurityPanel);
  const isTrashOpen = useUIStore((s) => s.isTrashOpen);
  const openTrash = useUIStore((s) => s.openTrash);
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);

  const handleSelectGroup = (id: string | null) => {
    setSelectedGroup(id);
    loadItems(id || undefined);
  };

  const handleSelectFavorites = () => {
    setShowFavoritesOnly(true);
    loadItems(undefined);
  };

  const handleSelectTag = (id: string) => {
    setSelectedTag(id);
    void loadItemsByTag(id);
  };

  return (
    <div className="flex h-full w-60 flex-col border-r border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-800">
      <div className="p-4">
        <input
          type="text"
          placeholder={t('sidebar.searchPlaceholder')}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
        />
      </div>
      
      <div className="flex-1 overflow-y-auto px-2">
        <button
          onClick={() => handleSelectGroup(null)}
          className={`w-full rounded-md px-3 py-2 text-left text-sm ${
            selectedGroupId === null && !showFavoritesOnly && !isTrashOpen
              ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-100' 
              : 'hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300'
          }`}
        >
          📁 {t('sidebar.allItems')}
        </button>
        
        <button
          onClick={handleSelectFavorites}
          className={`w-full rounded-md px-3 py-2 text-left text-sm ${
            showFavoritesOnly 
              ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-100' 
              : 'hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300'
          }`}
        >
          ⭐ {t('sidebar.favorites')}
        </button>

        <button
          onClick={openTrash}
          className={`flex w-full items-center gap-1 rounded-md px-3 py-2 text-left text-sm ${
            isTrashOpen
              ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-100'
              : 'hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300'
          }`}
        >
          <Trash2 className="h-4 w-4 shrink-0" />
          <span className="flex-1">{t('sidebar.trash')}</span>
          {trashCount > 0 && (
            <span className="rounded-full bg-gray-200 px-1.5 text-xs dark:bg-gray-600 dark:text-gray-200">
              {trashCount}
            </span>
          )}
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
                {group.icon || '📂'} {groupDisplayName(group, t)}
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
                      {t('sidebar.rename')}
                    </button>
                    <button
                      onClick={() => {
                        openDeleteConfirm('group', group.id, groupDisplayName(group, t));
                        setMenuOpenId(null);
                      }}
                      className="block w-full px-3 py-2 text-left text-sm text-red-500 hover:bg-gray-100 dark:hover:bg-gray-600"
                    >
                      {t('sidebar.delete')}
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
          <Plus className="mr-2 h-4 w-4" /> {t('sidebar.addGroup')}
        </Button>

        <div className="my-2 border-t border-gray-200 dark:border-gray-700" />

        <div className="px-3 py-1 text-xs uppercase tracking-wide text-gray-400">
          {t('tags.title')}
        </div>
        {tags.map((tag) => (
          <div key={tag.id} className="group flex items-center">
            <button
              onClick={() => handleSelectTag(tag.id)}
              className={`flex-1 rounded-md px-3 py-1.5 text-left text-sm ${
                selectedTagId === tag.id
                  ? 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-100'
                  : 'hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300'
              }`}
            >
              # {tag.name}
            </button>
            <button
              onClick={() => {
                if (window.confirm(t('tags.deleteConfirm', { name: tag.name }))) {
                  void useVaultStore.getState().deleteTag(tag.id);
                  if (selectedTagId === tag.id) setSelectedTag(null);
                }
              }}
              className="rounded p-1 text-gray-400 opacity-0 group-hover:opacity-100 hover:bg-gray-200 hover:text-red-500 dark:hover:bg-gray-600"
              title={t('common.delete')}
            >
              ×
            </button>
          </div>
        ))}
        <Button
          variant="ghost"
          size="sm"
          className="w-full justify-start text-gray-500 dark:text-gray-400"
          onClick={openAddTagModal}
        >
          <Plus className="mr-2 h-4 w-4" /> {t('tags.add')}
        </Button>
      </div>
      
      <div className="border-t border-gray-200 p-2 dark:border-gray-700">
        <Button
          variant="ghost"
          size="sm"
          className="w-full justify-start text-gray-500 dark:text-gray-400"
          onClick={openSecurityPanel}
        >
          <ShieldCheck className="mr-2 h-4 w-4" /> {t('sidebar.securityCheck')}
        </Button>
        <ImportExportModal />
        <ChangePasswordModal />
        <SettingsModal />
        <div className="mt-2 flex items-center justify-between">
          <Button
            variant="ghost"
            size="sm"
            className="text-gray-500 dark:text-gray-400"
            onClick={lock}
          >
            <Lock className="mr-2 h-4 w-4" /> {t('sidebar.lock')}
          </Button>
          <ThemeToggle />
        </div>
      </div>
    </div>
  );
}
