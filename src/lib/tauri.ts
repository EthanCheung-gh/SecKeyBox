import { invoke } from '@tauri-apps/api/core';
import type { Group, ItemSummary, ItemDetail, SecurityAudit, ImportToolsResult, TotpCode } from '@/types';

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

export async function changeMasterPassword(
  currentPassword: string,
  newPassword: string
): Promise<void> {
  return invoke<void>('change_master_password', { currentPassword, newPassword });
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

/**
 * 全库搜索：后端 LIKE 匹配 title / subtitle / username 类字段。
 *
 * 锁定态语义：后端返回空列表而非错误；调用方（vault store）负责用
 * isUnlocked 状态区分「已锁定」与「无结果」，锁定时不应发起本调用。
 */
export async function searchItems(query: string): Promise<ItemSummary[]> {
  return invoke<ItemSummary[]>('search_items', { query });
}

export async function getItemsByGroup(groupId: string): Promise<ItemSummary[]> {
  return invoke<ItemSummary[]>('get_items_by_group_cmd', { group_id: groupId });
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
  return invoke<string>('create_new_account_item', { group_id: groupId, title, username, password, website, notes });
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
  console.log('[DEBUG] createNewApiKeyItem called with:', { groupId, title, keyName });
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

export async function createNewEnvVarItem(
  groupId: string,
  title: string,
  variables: { key: string; value: string }[],
  notes?: string
): Promise<string> {
  return invoke<string>('create_new_env_var_item', {
    group_id: groupId,
    title,
    variables,
    notes
  });
}

export async function updateExistingEnvVarItem(
  id: string,
  title: string,
  variables: { key: string; value: string }[],
  notes?: string
): Promise<void> {
  return invoke<void>('update_existing_env_var_item', {
    id,
    title,
    variables,
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

export async function getTotpCode(id: string): Promise<TotpCode> {
  return invoke<TotpCode>('get_totp_code', { id });
}

export async function runSecurityAudit(): Promise<SecurityAudit> {
  return invoke<SecurityAudit>('security_audit');
}

export async function importCsv(data: string, groupId: string): Promise<ImportToolsResult> {
  return invoke<ImportToolsResult>('import_csv', { data, groupId });
}

export async function importEnv(groupId: string, title: string, content: string): Promise<ImportToolsResult> {
  return invoke<ImportToolsResult>('import_env', { groupId, title, content });
}

export interface ImportResult {
  groups_imported: number;
  items_imported: number;
  groups_skipped: number;
  items_skipped: number;
}

export async function exportVault(): Promise<string> {
  return invoke<string>('export_vault');
}

export async function importVault(data: string, mode: 'merge' | 'replace'): Promise<ImportResult> {
  return invoke<ImportResult>('import_vault', { data, mode });
}