import { invoke } from '@tauri-apps/api/core';
import type { Group, ItemSummary, ItemDetail, SecurityAudit, ImportToolsResult, TotpCode, SecuritySettings, AttachmentMeta, AttachmentData, AuditEntry, CustomField, Tag, BackupInfo } from '@/types';

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

/** Optional context so the backend can record an activity entry. */
export interface CopyContext {
  itemId?: string;
  itemTitle?: string;
  field?: string;
}

export async function copyToClipboard(text: string, context?: CopyContext): Promise<void> {
  return invoke<void>('copy_to_clipboard', {
    text,
    itemId: context?.itemId,
    itemTitle: context?.itemTitle,
    field: context?.field,
  });
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

// --- Security policy (idle auto-lock) ---

/** Tell the backend the user is active; throttled by the caller. */
export async function recordActivity(): Promise<void> {
  return invoke<void>('record_activity');
}

export async function getSecuritySettings(): Promise<SecuritySettings> {
  return invoke<SecuritySettings>('get_security_settings');
}

/** `0` disables the idle lock. */
export async function setAutoLockMinutes(minutes: number): Promise<void> {
  return invoke<void>('set_auto_lock_minutes', { minutes });
}

// --- Attachments (encrypted with the vault key) ---

export async function listAttachments(itemId: string): Promise<AttachmentMeta[]> {
  return invoke<AttachmentMeta[]>('list_attachments_cmd', { itemId });
}

export async function addAttachment(
  itemId: string,
  filename: string,
  contentType: string | undefined,
  dataBase64: string
): Promise<AttachmentMeta> {
  return invoke<AttachmentMeta>('add_attachment_cmd', {
    itemId,
    filename,
    contentType,
    dataBase64,
  });
}

export async function getAttachment(
  itemId: string,
  attachmentId: string
): Promise<AttachmentData> {
  return invoke<AttachmentData>('get_attachment_cmd', { itemId, attachmentId });
}

export async function saveAttachmentToPath(
  itemId: string,
  attachmentId: string,
  path: string
): Promise<void> {
  return invoke<void>('save_attachment_to_path', { itemId, attachmentId, path });
}

export async function deleteAttachment(itemId: string, attachmentId: string): Promise<void> {
  return invoke<void>('delete_attachment_cmd', { itemId, attachmentId });
}

// --- Activity log ---

export async function getActivityLog(limit = 200): Promise<AuditEntry[]> {
  return invoke<AuditEntry[]>('get_activity_log', { limit });
}

// --- Custom fields & tags ---

export async function listCustomFields(itemId: string): Promise<CustomField[]> {
  return invoke<CustomField[]>('list_custom_fields_cmd', { itemId });
}

export async function saveCustomFields(
  itemId: string,
  fields: { key: string; value: string }[]
): Promise<void> {
  return invoke<void>('save_custom_fields_cmd', { itemId, fields });
}

export async function getTags(): Promise<Tag[]> {
  return invoke<Tag[]>('get_tags');
}

export async function createTag(name: string, color?: string): Promise<Tag> {
  return invoke<Tag>('create_new_tag', { name, color });
}

export async function deleteTag(id: string): Promise<void> {
  return invoke<void>('delete_existing_tag', { id });
}

export async function getItemTags(itemId: string): Promise<string[]> {
  return invoke<string[]>('get_item_tags', { itemId });
}

export async function setItemTags(itemId: string, tagIds: string[]): Promise<void> {
  return invoke<void>('set_item_tags_cmd', { itemId, tagIds });
}

export async function getItemsByTag(tagId: string): Promise<ItemSummary[]> {
  return invoke<ItemSummary[]>('get_items_by_tag_cmd', { tagId });
}

// --- Local backups ---

export async function createBackup(): Promise<BackupInfo> {
  return invoke<BackupInfo>('create_backup');
}

export async function listBackups(): Promise<BackupInfo[]> {
  return invoke<BackupInfo[]>('list_backups');
}

export async function deleteBackup(file: string): Promise<void> {
  return invoke<void>('delete_backup', { file });
}

export async function restoreBackup(file: string): Promise<void> {
  return invoke<void>('restore_backup', { file });
}