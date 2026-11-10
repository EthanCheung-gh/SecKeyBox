import { create } from 'zustand';

interface UIState {
  selectedGroupId: string | null;
  selectedTagId: string | null;
  showFavoritesOnly: boolean;
  searchQuery: string;
  sortBy: 'name_asc' | 'name_desc' | 'created' | 'updated';
  isAddItemModalOpen: boolean;
  isAddGroupModalOpen: boolean;
  isAddTagModalOpen: boolean;
  editingGroupId: string | null;
  editingItemId: string | null;
  deleteConfirmTarget: { type: 'group' | 'item'; id: string; name: string } | null;
  newItemType: 'account' | 'api_key' | 'env_var' | 'database' | 'ssh' | 'cloud' | 'license' | 'smtp';
  isSecurityPanelOpen: boolean;
  isTrashOpen: boolean;

  setSelectedGroup: (id: string | null) => void;
  setSelectedTag: (id: string | null) => void;
  setShowFavoritesOnly: (show: boolean) => void;
  setSearchQuery: (query: string) => void;
  setSortBy: (sort: UIState['sortBy']) => void;
  openSecurityPanel: () => void;
  closeSecurityPanel: () => void;
  openTrash: () => void;
  closeTrash: () => void;
  openAddItemModal: () => void;
  closeAddItemModal: () => void;
  openAddGroupModal: () => void;
  openAddTagModal: () => void;
  closeAddTagModal: () => void;
  closeAddGroupModal: () => void;
  openEditGroupModal: (id: string) => void;
  closeEditGroupModal: () => void;
  openEditItemModal: (id: string) => void;
  closeEditItemModal: () => void;
  openDeleteConfirm: (type: 'group' | 'item', id: string, name: string) => void;
  closeDeleteConfirm: () => void;
  setNewItemType: (type: UIState['newItemType']) => void;
}

export const useUIStore = create<UIState>((set) => ({
  selectedGroupId: null,
  selectedTagId: null,
  showFavoritesOnly: false,
  searchQuery: '',
  sortBy: 'name_asc',
  isAddItemModalOpen: false,
  isAddGroupModalOpen: false,
  isAddTagModalOpen: false,
  editingGroupId: null,
  editingItemId: null,
  deleteConfirmTarget: null,
  newItemType: 'account',
  isSecurityPanelOpen: false,
  isTrashOpen: false,

  setSelectedGroup: (id) => set({ selectedGroupId: id, selectedTagId: null, showFavoritesOnly: false, isTrashOpen: false, isSecurityPanelOpen: false }),
  setSelectedTag: (id) => set({ selectedTagId: id, selectedGroupId: null, showFavoritesOnly: false, isTrashOpen: false, isSecurityPanelOpen: false }),
  setShowFavoritesOnly: (show) => set({ showFavoritesOnly: show, selectedGroupId: null, selectedTagId: null, isTrashOpen: false, isSecurityPanelOpen: false }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSortBy: (sort) => set({ sortBy: sort }),
  openAddItemModal: () => set({ isAddItemModalOpen: true }),
  closeAddItemModal: () => set({ isAddItemModalOpen: false }),
  openAddGroupModal: () => set({ isAddGroupModalOpen: true }),
  closeAddGroupModal: () => set({ isAddGroupModalOpen: false }),
  openAddTagModal: () => set({ isAddTagModalOpen: true }),
  closeAddTagModal: () => set({ isAddTagModalOpen: false }),
  openEditGroupModal: (id) => set({ editingGroupId: id }),
  closeEditGroupModal: () => set({ editingGroupId: null }),
  openEditItemModal: (id) => set({ editingItemId: id }),
  closeEditItemModal: () => set({ editingItemId: null }),
  openDeleteConfirm: (type, id, name) => set({ deleteConfirmTarget: { type, id, name } }),
  closeDeleteConfirm: () => set({ deleteConfirmTarget: null }),
  setNewItemType: (type) => set({ newItemType: type }),
  openSecurityPanel: () => set({ isSecurityPanelOpen: true, isTrashOpen: false }),
  closeSecurityPanel: () => set({ isSecurityPanelOpen: false }),
  // Trash reuses the item list area: opening it hides the security panel but
  // keeps the group/tag/favorites selection so closing it returns there.
  openTrash: () => set({ isTrashOpen: true, isSecurityPanelOpen: false }),
  closeTrash: () => set({ isTrashOpen: false }),
}));