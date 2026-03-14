import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Group, ItemSummary, AccountItemDetail, ApiKeyItemDetail, EnvVarItemDetail, EnvVarPair } from '@/types';
import * as api from '@/lib/tauri'; // 保留读取操作的 api 调用

interface VaultState {
  isInitialized: boolean | null;
  isUnlocked: boolean;
  groups: Group[];
  items: ItemSummary[];
  selectedItem: AccountItemDetail | ApiKeyItemDetail | EnvVarItemDetail | null;
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
  createApiKeyItem: (data: {
    groupId: string;
    title: string;
    keyName: string;
    keyValue: string;
    endpoint?: string;
    authMethod?: string;
    notes?: string;
  }) => Promise<void>;
  updateApiKeyItem: (id: string, data: {
    title: string;
    keyName: string;
    keyValue?: string;
    endpoint?: string;
    authMethod?: string;
    notes?: string;
  }) => Promise<void>;
  createEnvVarItem: (data: {
    groupId: string;
    title: string;
    variables: EnvVarPair[];
    notes?: string;
  }) => Promise<void>;
  updateEnvVarItem: (id: string, data: {
    title: string;
    variables: EnvVarPair[];
    notes?: string;
  }) => Promise<void>;
  toggleItemFavorite: (id: string) => Promise<void>;
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

  // --- 这里的读取操作保留 api 封装，因为它们没有报错 ---
  checkInitialized: async () => {
    try {
      const initialized = await api.isVaultInitialized();
      set({ isInitialized: initialized });
    } catch (e) {
      console.error('checkInitialized error:', e);
      set({ isInitialized: false, error: String(e) });
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
      set({ selectedItem: item as AccountItemDetail | ApiKeyItemDetail | EnvVarItemDetail });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearSelection: () => set({ selectedItem: null }),

  // --- 从这里开始，所有的写入操作全改为直接 invoke ---

  createGroup: async (name, icon) => {
    try {
      await invoke('create_new_group', { name, icon });
      await get().loadGroups();
    } catch (e) {
      console.error('[DEBUG] createGroup error:', e);
      set({ error: String(e) });
    }
  },

  updateGroup: async (id, name, icon) => {
    try {
      await invoke('update_existing_group', { id, name, icon });
      await get().loadGroups();
    } catch (e) {
      console.error('[DEBUG] updateGroup error:', e);
      set({ error: String(e) });
    }
  },

  deleteGroup: async (id) => {
    try {
      await invoke('delete_existing_group', { id });
      // Immediately remove items of this group from local state
      set((state) => ({
        items: state.items.filter(item => item.group_id !== id)
      }));
      await get().loadGroups();
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] deleteGroup error:', e);
      set({ error: String(e) });
    }
  },

  createItem: async (data) => {
    try {
      await invoke('create_new_account_item', {
        groupId: data.groupId, 
        title: data.title,
        username: data.username,
        password: data.password,
        website: data.website,
        notes: data.notes
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createItem error:', e);
      set({ error: String(e) });
    }
  },

  updateItem: async (id, data) => {
    try {
      await invoke('update_existing_account_item', {
        id,
        title: data.title,
        username: data.username,
        password: data.password,
        website: data.website,
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateItem error:', e);
      set({ error: String(e) });
    }
  },

  deleteItem: async (id) => {
    try {
      await invoke('delete_existing_item', { id });
      // Immediately remove from local state for instant UI update
      set((state) => ({
        items: state.items.filter(item => item.id !== id),
        selectedItem: state.selectedItem?.id === id ? null : state.selectedItem
      }));
      // Then reload from backend to ensure consistency
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] deleteItem error:', e);
      set({ error: String(e) });
    }
  },

  createApiKeyItem: async (data) => {
    try {
      await invoke('create_new_api_key_item', {
        request: {
          group_id: data.groupId,
          title: data.title,
          key_name: data.keyName,
          key_value: data.keyValue,
          endpoint: data.endpoint,
          auth_method: data.authMethod,
          notes: data.notes
        }
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createApiKeyItem error:', e);
      set({ error: String(e) });
    }
  },

  updateApiKeyItem: async (id, data) => {
    try {
      // 保持和 createApiKey 一致的结构体模式
      await invoke('update_existing_api_key_item', {
        request: {
          id: id,
          title: data.title,
          key_name: data.keyName,
          key_value: data.keyValue,
          endpoint: data.endpoint,
          auth_method: data.authMethod,
          notes: data.notes
        }
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateApiKeyItem error:', e);
      set({ error: String(e) });
    }
  },

  createEnvVarItem: async (data) => {
    try {
      await invoke('create_new_env_var_item', {
        groupId: data.groupId,
        title: data.title,
        variables: data.variables,
        notes: data.notes
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createEnvVarItem error:', e);
      set({ error: String(e) });
    }
  },

  updateEnvVarItem: async (id, data) => {
    try {
      await invoke('update_existing_env_var_item', {
        id, 
        title: data.title, 
        variables: data.variables, 
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateEnvVarItem error:', e);
      set({ error: String(e) });
    }
  },

  toggleItemFavorite: async (id) => {
    try {
      await invoke('toggle_item_favorite', { id });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] toggleItemFavorite error:', e);
      set({ error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
}));