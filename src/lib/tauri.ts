import { invoke } from '@tauri-apps/api/core';
import type { Group, ItemSummary, ItemDetail } from '@/types';

export async function isVaultInitialized(): Promise<boolean> {
  return invoke<boolean>('is_vault_initialized');
}

export async function initializeVault(masterPassword: string): Promise<void> {
  return invoke<void>('initialize_vault', { masterPassword });
}

export async function unlockVault(masterPassword: string): Promise<boolean> {
  return invoke<boolean>('unlock_vault', { masterPassword });
}

export async function lockVault(): Promise<void> {
  return invoke<void>('lock_vault');
}

export async function isVaultUnlocked(): Promise<boolean> {
  return invoke<boolean>('is_vault_unlocked');
}

export async function getGroups(): Promise<Group[]> {
  return invoke<Group[]>('get_groups');
}

export async function createNewGroup(name: string, icon?: string, parentId?: string, sortOrder?: number): Promise<Group> {
  return invoke<Group>('create_new_group', { name, icon, parentId, sortOrder });
}

export async function updateExistingGroup(id: string, name: string, icon?: string, sortOrder?: number): Promise<void> {
  return invoke<void>('update_existing_group', { id, name, icon, sortOrder });
}

export async function deleteExistingGroup(id: string): Promise<void> {
  return invoke<void>('delete_existing_group', { id });
}

export async function getAllItems(): Promise<ItemSummary[]> {
  return invoke<ItemSummary[]>('get_all_items_cmd');
}

export async function getItemsByGroup(groupId: string): Promise<ItemSummary[]> {
  return invoke<ItemSummary[]>('get_items_by_group_cmd', { groupId });
}

export async function getItemDetail(id: string): Promise<ItemDetail> {
  return invoke<ItemDetail>('get_item_detail', { id });
}

export async function createNewAccountItem(
  groupId: string,
  title: string,
  username: string,
  password: string,
  website?: string,
  notes?: string
): Promise<string> {
  return invoke<string>('create_new_account_item', { groupId, title, username, password, website, notes });
}

export async function updateExistingAccountItem(
  id: string,
  title: string,
  username: string,
  password?: string,
  website?: string,
  notes?: string
): Promise<void> {
  return invoke<void>('update_existing_account_item', { id, title, username, password, website, notes });
}

export async function createNewApiKeyItem(
  groupId: string,
  title: string,
  keyName: string,
  keyValue: string,
  endpoint?: string,
  authMethod?: string,
  notes?: string
): Promise<string> {
  return invoke<string>('create_new_api_key_item', {
    group_id: groupId,
    title,
    key_name: keyName,
    key_value: keyValue,
    endpoint,
    auth_method: authMethod,
    notes
  });
}

export async function updateExistingApiKeyItem(
  id: string,
  title: string,
  keyName: string,
  keyValue?: string,
  endpoint?: string,
  authMethod?: string,
  notes?: string
): Promise<void> {
  return invoke<void>('update_existing_api_key_item', {
    id,
    title,
    key_name: keyName,
    key_value: keyValue,
    endpoint,
    auth_method: authMethod,
    notes
  });
}

export async function deleteExistingItem(id: string): Promise<void> {
  return invoke<void>('delete_existing_item', { id });
}

export async function toggleItemFavorite(id: string): Promise<void> {
  return invoke<void>('toggle_item_favorite', { id });
}

export async function copyToClipboard(text: string): Promise<void> {
  return invoke<void>('copy_to_clipboard', { text });
}

export async function clearClipboard(): Promise<void> {
  return invoke<void>('clear_clipboard');
}