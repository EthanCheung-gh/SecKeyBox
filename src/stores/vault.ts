import { create } from 'zustand';
import type { Group, ItemSummary, ItemDetail } from '@/types';
import * as api from '@/lib/tauri';

interface VaultState {
  isInitialized: boolean | null;
  isUnlocked: boolean;
  groups: Group[];
  items: ItemSummary[];
  selectedItem: ItemDetail | null;
  isLoading: boolean;
  error: string | null;
  
  checkInitialized: () => Promise<void>;
  initialize: (password: string) => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  loadGroups: () => Promise<void>;
  loadItems: (groupId?: string) => Promise<void>;
  selectItem: (id: string) => Promise<void>;
  clearSelection: () => void;
  createGroup: (name: string, icon?: string) => Promise<void>;
  updateGroup: (id: string, name: string, icon?: string) => Promise<void>;
  deleteGroup: (id: string) => Promise<void>;
  createItem: (data: {
    groupId: string;
    title: string;
    username: string;
    password: string;
    website?: string;
    notes?: string;
  }) => Promise<void>;
  updateItem: (id: string, data: {
    title: string;
    username: string;
    password?: string;
    website?: string;
    notes?: string;
  }) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  clearError: () => void;
}

export const useVaultStore = create<VaultState>((set, get) => ({
  isInitialized: null,
  isUnlocked: false,
  groups: [],
  items: [],
  selectedItem: null,
  isLoading: false,
  error: null,

  checkInitialized: async () => {
    try {
      const initialized = await api.isVaultInitialized();
      set({ isInitialized: initialized });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  initialize: async (password) => {
    set({ isLoading: true, error: null });
    try {
      await api.initializeVault(password);
      set({ isInitialized: true, isUnlocked: true, isLoading: false });
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  unlock: async (password) => {
    set({ isLoading: true, error: null });
    try {
      await api.unlockVault(password);
      set({ isUnlocked: true, isLoading: false });
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  lock: async () => {
    await api.lockVault();
    set({ isUnlocked: false, items: [], selectedItem: null });
  },

  loadGroups: async () => {
    try {
      const groups = await api.getGroups();
      set({ groups });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadItems: async (groupId) => {
    try {
      const items = groupId 
        ? await api.getItemsByGroup(groupId)
        : await api.getAllItems();
      set({ items });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  selectItem: async (id) => {
    try {
      const item = await api.getItemDetail(id);
      set({ selectedItem: item });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearSelection: () => set({ selectedItem: null }),

  createGroup: async (name, icon) => {
    try {
      await api.createNewGroup(name, icon);
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateGroup: async (id, name, icon) => {
    try {
      await api.updateExistingGroup(id, name, icon);
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteGroup: async (id) => {
    try {
      await api.deleteExistingGroup(id);
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createItem: async (data) => {
    try {
      await api.createNewAccountItem(
        data.groupId,
        data.title,
        data.username,
        data.password,
        data.website,
        data.notes
      );
      await get().loadItems();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateItem: async (id, data) => {
    try {
      await api.updateExistingAccountItem(
        id,
        data.title,
        data.username,
        data.password,
        data.website,
        data.notes
      );
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteItem: async (id) => {
    try {
      await api.deleteExistingItem(id);
      if (get().selectedItem?.id === id) {
        set({ selectedItem: null });
      }
      await get().loadItems();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
}));