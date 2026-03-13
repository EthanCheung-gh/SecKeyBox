import { create } from 'zustand';

interface UIState {
  selectedGroupId: string | null;
  showFavoritesOnly: boolean;
  searchQuery: string;
  sortBy: 'name_asc' | 'name_desc' | 'created' | 'updated';
  isAddItemModalOpen: boolean;
  isAddGroupModalOpen: boolean;
  editingGroupId: string | null;
  editingItemId: string | null;
  deleteConfirmTarget: { type: 'group' | 'item'; id: string; name: string } | null;
  newItemType: 'account' | 'api_key' | 'env_var';
  
  setSelectedGroup: (id: string | null) => void;
  setShowFavoritesOnly: (show: boolean) => void;
  setSearchQuery: (query: string) => void;
  setSortBy: (sort: UIState['sortBy']) => void;
  openAddItemModal: () => void;
  closeAddItemModal: () => void;
  openAddGroupModal: () => void;
  closeAddGroupModal: () => void;
  openEditGroupModal: (id: string) => void;
  closeEditGroupModal: () => void;
  openEditItemModal: (id: string) => void;
  closeEditItemModal: () => void;
  openDeleteConfirm: (type: 'group' | 'item', id: string, name: string) => void;
  closeDeleteConfirm: () => void;
  setNewItemType: (type: 'account' | 'api_key' | 'env_var') => void;
}

export const useUIStore = create<UIState>((set) => ({
  selectedGroupId: null,
  showFavoritesOnly: false,
  searchQuery: '',
  sortBy: 'name_asc',
  isAddItemModalOpen: false,
  isAddGroupModalOpen: false,
  editingGroupId: null,
  editingItemId: null,
  deleteConfirmTarget: null,
  newItemType: 'account',

  setSelectedGroup: (id) => set({ selectedGroupId: id, showFavoritesOnly: false }),
  setShowFavoritesOnly: (show) => set({ showFavoritesOnly: show, selectedGroupId: null }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSortBy: (sort) => set({ sortBy: sort }),
  openAddItemModal: () => set({ isAddItemModalOpen: true }),
  closeAddItemModal: () => set({ isAddItemModalOpen: false }),
  openAddGroupModal: () => set({ isAddGroupModalOpen: true }),
  closeAddGroupModal: () => set({ isAddGroupModalOpen: false }),
  openEditGroupModal: (id) => set({ editingGroupId: id }),
  closeEditGroupModal: () => set({ editingGroupId: null }),
  openEditItemModal: (id) => set({ editingItemId: id }),
  closeEditItemModal: () => set({ editingItemId: null }),
  openDeleteConfirm: (type, id, name) => set({ deleteConfirmTarget: { type, id, name } }),
  closeDeleteConfirm: () => set({ deleteConfirmTarget: null }),
  setNewItemType: (type) => set({ newItemType: type }),
}));