import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Group, ItemSummary, AccountItemDetail, ApiKeyItemDetail, EnvVarItemDetail, DatabaseItemDetail, SshItemDetail, CloudItemDetail, LicenseItemDetail, SmtpItemDetail, EnvVarPair } from '@/types';
import * as api from '@/lib/tauri'; // 保留读取操作的 api 调用

interface VaultState {
  isInitialized: boolean | null;
  isUnlocked: boolean;
  groups: Group[];
  items: ItemSummary[];
  /**
   * search_items 的最新结果（带触发时的 query 快照，用于丢弃过期响应）。
   * null 表示当前不在后端搜索状态（未输入搜索词，或已锁定/已清除）。
   */
  searchResults: { query: string; items: ItemSummary[] } | null;
  selectedItem: AccountItemDetail | ApiKeyItemDetail | EnvVarItemDetail | DatabaseItemDetail | SshItemDetail | CloudItemDetail | LicenseItemDetail | SmtpItemDetail | null;
  isLoading: boolean;
  error: string | null;

  checkInitialized: () => Promise<void>;
  initialize: (password: string) => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  loadGroups: () => Promise<void>;
  loadItems: (groupId?: string) => Promise<void>;
  searchItems: (query: string) => Promise<void>;
  clearSearch: () => void;
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
  createDatabaseItem: (data: {
    groupId: string;
    title: string;
    dbType: string;
    host: string;
    port?: number;
    databaseName?: string;
    username?: string;
    password?: string;
    connectionUrl?: string;
    notes?: string;
  }) => Promise<void>;
  updateDatabaseItem: (id: string, data: {
    title: string;
    dbType: string;
    host: string;
    port?: number;
    databaseName?: string;
    username?: string;
    password?: string;
    connectionUrl?: string;
    notes?: string;
  }) => Promise<void>;
  createSshItem: (data: {
    groupId: string;
    title: string;
    host: string;
    port?: number;
    username: string;
    password?: string;
    keyPath?: string;
    passphrase?: string;
    notes?: string;
  }) => Promise<void>;
  updateSshItem: (id: string, data: {
    title: string;
    host: string;
    port?: number;
    username: string;
    password?: string;
    keyPath?: string;
    passphrase?: string;
    notes?: string;
  }) => Promise<void>;
  createCloudItem: (data: {
    groupId: string;
    title: string;
    provider: string;
    accessKeyId: string;
    secret: string;
    region?: string;
    notes?: string;
  }) => Promise<void>;
  updateCloudItem: (id: string, data: {
    title: string;
    provider: string;
    accessKeyId: string;
    secret?: string;
    region?: string;
    notes?: string;
  }) => Promise<void>;
  createLicenseItem: (data: {
    groupId: string;
    title: string;
    softwareName: string;
    licenseKey: string;
    boundEmail?: string;
    expiryDate?: number;
    notes?: string;
  }) => Promise<void>;
  updateLicenseItem: (id: string, data: {
    title: string;
    softwareName: string;
    licenseKey?: string;
    boundEmail?: string;
    expiryDate?: number;
    notes?: string;
  }) => Promise<void>;
  createSmtpItem: (data: {
    groupId: string;
    title: string;
    host: string;
    port?: number;
    encryption?: string;
    username?: string;
    password: string;
    fromAddress?: string;
    notes?: string;
  }) => Promise<void>;
  updateSmtpItem: (id: string, data: {
    title: string;
    host: string;
    port?: number;
    encryption?: string;
    username?: string;
    password?: string;
    fromAddress?: string;
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
  searchResults: null,
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
    // 同步清除搜索状态，避免锁定后残留旧结果
    set({ isUnlocked: false, items: [], selectedItem: null, searchResults: null });
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

  /**
   * 调用后端 search_items 命令（LIKE 匹配 title/subtitle/username）。
   *
   * 锁定态守卫：锁定时不发起查询（后端此时只会返回空列表），
   * 以便 UI 能区分「已锁定」与「已解锁但无搜索结果」两种情况。
   */
  searchItems: async (query) => {
    if (!get().isUnlocked) {
      set({ searchResults: null });
      return;
    }
    try {
      const items = await api.searchItems(query);
      set({ searchResults: { query, items } });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearSearch: () => set({ searchResults: null }),

  selectItem: async (id) => {
    try {
      const item = await api.getItemDetail(id);
      set({ selectedItem: item });
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
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
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
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
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
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  updateApiKeyItem: async (id, data) => {
    try {
      // 后端签名是平铺参数（id, title, key_name, ...），不要用 request 包装
      await invoke('update_existing_api_key_item', {
        id: id,
        title: data.title,
        keyName: data.keyName,
        keyValue: data.keyValue,
        endpoint: data.endpoint,
        authMethod: data.authMethod,
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateApiKeyItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
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
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
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
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  createDatabaseItem: async (data) => {
    try {
      await invoke('create_new_database_item', {
        groupId: data.groupId,
        title: data.title,
        dbType: data.dbType,
        host: data.host,
        port: data.port,
        databaseName: data.databaseName,
        username: data.username,
        password: data.password,
        connectionUrl: data.connectionUrl,
        notes: data.notes
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createDatabaseItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  updateDatabaseItem: async (id, data) => {
    try {
      await invoke('update_existing_database_item', {
        id,
        title: data.title,
        dbType: data.dbType,
        host: data.host,
        port: data.port,
        databaseName: data.databaseName,
        username: data.username,
        password: data.password,
        connectionUrl: data.connectionUrl,
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateDatabaseItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  createSshItem: async (data) => {
    try {
      await invoke('create_new_ssh_item', {
        groupId: data.groupId,
        title: data.title,
        host: data.host,
        port: data.port,
        username: data.username,
        password: data.password,
        keyPath: data.keyPath,
        passphrase: data.passphrase,
        notes: data.notes
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createSshItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  updateSshItem: async (id, data) => {
    try {
      await invoke('update_existing_ssh_item', {
        id,
        title: data.title,
        host: data.host,
        port: data.port,
        username: data.username,
        password: data.password,
        keyPath: data.keyPath,
        passphrase: data.passphrase,
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateSshItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  createCloudItem: async (data) => {
    try {
      await invoke('create_new_cloud_item', {
        groupId: data.groupId,
        title: data.title,
        provider: data.provider,
        accessKeyId: data.accessKeyId,
        secret: data.secret,
        region: data.region,
        notes: data.notes
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createCloudItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  updateCloudItem: async (id, data) => {
    try {
      await invoke('update_existing_cloud_item', {
        id,
        title: data.title,
        provider: data.provider,
        accessKeyId: data.accessKeyId,
        secret: data.secret,
        region: data.region,
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateCloudItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  createLicenseItem: async (data) => {
    try {
      await invoke('create_new_license_item', {
        groupId: data.groupId,
        title: data.title,
        softwareName: data.softwareName,
        licenseKey: data.licenseKey,
        boundEmail: data.boundEmail,
        expiryDate: data.expiryDate,
        notes: data.notes
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createLicenseItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  updateLicenseItem: async (id, data) => {
    try {
      await invoke('update_existing_license_item', {
        id,
        title: data.title,
        softwareName: data.softwareName,
        licenseKey: data.licenseKey,
        boundEmail: data.boundEmail,
        expiryDate: data.expiryDate,
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateLicenseItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  createSmtpItem: async (data) => {
    try {
      await invoke('create_new_smtp_item', {
        groupId: data.groupId,
        title: data.title,
        host: data.host,
        port: data.port,
        encryption: data.encryption,
        username: data.username,
        password: data.password,
        fromAddress: data.fromAddress,
        notes: data.notes
      });
      await get().loadItems();
    } catch (e) {
      console.error('[DEBUG] createSmtpItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
    }
  },

  updateSmtpItem: async (id, data) => {
    try {
      await invoke('update_existing_smtp_item', {
        id,
        title: data.title,
        host: data.host,
        port: data.port,
        encryption: data.encryption,
        username: data.username,
        password: data.password,
        fromAddress: data.fromAddress,
        notes: data.notes
      });
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      console.error('[DEBUG] updateSmtpItem error:', e);
      set({ error: String(e) });
  
      // 向调用方（弹窗）传播失败，避免“提交失败却关闭弹窗”的无感知体验
      throw e;
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