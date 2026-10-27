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

  setSelectedGroup: (id: string | null) => void;
  setSelectedTag: (id: string | null) => void;
  setShowFavoritesOnly: (show: boolean) => void;
  setSearchQuery: (query: string) => void;
  setSortBy: (sort: UIState['sortBy']) => void;
  openSecurityPanel: () => void;
  closeSecurityPanel: () => void;
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

  setSelectedGroup: (id) => set({ selectedGroupId: id, selectedTagId: null, showFavoritesOnly: false }),
  setSelectedTag: (id) => set({ selectedTagId: id, selectedGroupId: null, showFavoritesOnly: false }),
  setShowFavoritesOnly: (show) => set({ showFavoritesOnly: show, selectedGroupId: null, selectedTagId: null }),
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
  openSecurityPanel: () => set({ isSecurityPanelOpen: true }),
  closeSecurityPanel: () => set({ isSecurityPanelOpen: false }),
}));