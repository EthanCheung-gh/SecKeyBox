import { invoke } from '@tauri-apps/api/core';

export async function isVaultInitialized(): Promise<boolean> {
  return invoke('is_vault_initialized');
}

export async function initializeVault(masterPassword: string): Promise<void> {
  return invoke('initialize_vault', { masterPassword });
}

export async function unlockVault(masterPassword: string): Promise<boolean> {
  return invoke('unlock_vault', { masterPassword });
}

export async function lockVault(): Promise<void> {
  return invoke('lock_vault');
}

export async function isVaultUnlocked(): Promise<boolean> {
  return invoke('is_vault_unlocked');
}

export async function getGroups() {
  return invoke('get_groups');
}

export async function createNewGroup(name: string, icon?: string, parentId?: string, sortOrder?: number) {
  return invoke('create_new_group', { name, icon, parentId, sortOrder });
}

export async function updateExistingGroup(id: string, name: string, icon?: string, sortOrder?: number) {
  return invoke('update_existing_group', { id, name, icon, sortOrder });
}

export async function deleteExistingGroup(id: string) {
  return invoke('delete_existing_group', { id });
}

export async function getAllItems() {
  return invoke('get_all_items_cmd');
}

export async function getItemsByGroup(groupId: string) {
  return invoke('get_items_by_group_cmd', { groupId });
}

export async function getItemDetail(id: string) {
  return invoke('get_item_detail', { id });
}

export async function createNewAccountItem(
  groupId: string,
  title: string,
  username: string,
  password: string,
  website?: string,
  notes?: string
) {
  return invoke('create_new_account_item', { groupId, title, username, password, website, notes });
}

export async function updateExistingAccountItem(
  id: string,
  title: string,
  username: string,
  password?: string,
  website?: string,
  notes?: string
) {
  return invoke('update_existing_account_item', { id, title, username, password, website, notes });
}

export async function deleteExistingItem(id: string) {
  return invoke('delete_existing_item', { id });
}

export async function toggleItemFavorite(id: string) {
  return invoke('toggle_item_favorite', { id });
}

export async function copyToClipboard(text: string) {
  return invoke('copy_to_clipboard', { text });
}

export async function clearClipboard() {
  return invoke('clear_clipboard');
}