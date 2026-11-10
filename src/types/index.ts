export interface Group {
  id: string;
  name: string;
  icon?: string;
  parent_id?: string;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export interface ItemSummary {
  id: string;
  title: string;
  subtitle: string;
  icon?: string;
  type: 'account' | 'api_key' | 'env_var' | 'database' | 'ssh' | 'cloud' | 'license' | 'smtp';
  is_favorite: boolean;
  group_id: string;
  created_at: number;
  updated_at: number;
}

/** One soft-deleted item as shown in the trash list. */
export interface TrashItemSummary {
  id: string;
  title: string;
  type: ItemSummary['type'];
  icon?: string;
  /** Unix seconds when the item was moved to the trash. */
  deleted_at: number;
  /** Group the item belonged to before it was trashed (kept after deletion). */
  original_group_id?: string | null;
  /** Name of `original_group_id` when that group still exists. */
  original_group_name?: string | null;
}

export interface SecretHistoryEntry {
  field: string;
  value: string;
  changed_at: number;
}

export interface AccountItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'account';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  username: string;
  password: string;
  website?: string;
  notes?: string;
  totp_secret?: string;
  secret_history: SecretHistoryEntry[];
}

export interface AuditFinding {
  item_id: string;
  title: string;
  detail: string;
}

export interface ReusedSecretGroup {
  secret_hint: string;
  items: AuditFinding[];
}

export interface SecurityAudit {
  weak_passwords: AuditFinding[];
  reused_secrets: ReusedSecretGroup[];
  stale_rotations: AuditFinding[];
  missing_2fa: AuditFinding[];
  expiring_licenses: AuditFinding[];
}

export interface ImportToolsResult {
  groups_imported: number;
  items_imported: number;
  items_skipped: number;
}

export interface TotpCode {
  code: string;
  seconds_remaining: number;
}

export interface ApiKeyItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'api_key';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  key_name: string;
  key_value: string;
  endpoint?: string;
  auth_method?: string;
  rotation_date?: number;
  notes?: string;
  secret_history: SecretHistoryEntry[];
}

export interface EnvVarPair {
  key: string;
  value: string;
}

export interface EnvVarItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'env_var';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  variables: EnvVarPair[];
  notes?: string;
}

export type ItemDetail = AccountItemDetail | ApiKeyItemDetail | EnvVarItemDetail | DatabaseItemDetail | SshItemDetail | CloudItemDetail | LicenseItemDetail | SmtpItemDetail;

export interface DatabaseItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'database';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  db_type: string;
  host: string;
  port?: number;
  database_name?: string;
  username?: string;
  password?: string;
  connection_url?: string;
  notes?: string;
  secret_history: SecretHistoryEntry[];
}

export interface SshItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'ssh';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  host: string;
  port?: number;
  username: string;
  password?: string;
  key_path?: string;
  passphrase?: string;
  notes?: string;
  secret_history: SecretHistoryEntry[];
}

export interface CloudItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'cloud';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  provider: string;
  access_key_id: string;
  secret: string;
  region?: string;
  notes?: string;
  secret_history: SecretHistoryEntry[];
}

export interface LicenseItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'license';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  software_name: string;
  license_key: string;
  bound_email?: string;
  expiry_date?: number;
  notes?: string;
  secret_history: SecretHistoryEntry[];
}

export interface SmtpItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'smtp';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  host: string;
  port?: number;
  encryption?: string;
  username?: string;
  password: string;
  from_address?: string;
  notes?: string;
  secret_history: SecretHistoryEntry[];
}

export interface CreateAccountItemInput {
  group_id: string;
  title: string;
  username: string;
  password: string;
  website?: string;
  notes?: string;
}

export interface UpdateAccountItemInput {
  id: string;
  title: string;
  username: string;
  password?: string;
  website?: string;
  notes?: string;
}

export interface CreateApiKeyItemInput {
  group_id: string;
  title: string;
  key_name: string;
  key_value: string;
  endpoint?: string;
  auth_method?: string;
  notes?: string;
}

export interface UpdateApiKeyItemInput {
  id: string;
  title: string;
  key_name: string;
  key_value?: string;
  endpoint?: string;
  auth_method?: string;
  notes?: string;
}

export interface CreateEnvVarItemInput {
  group_id: string;
  title: string;
  variables: EnvVarPair[];
  notes?: string;
}

export interface UpdateEnvVarItemInput {
  id: string;
  title: string;
  variables: EnvVarPair[];
  notes?: string;
}

export interface CreateGroupInput {
  name: string;
  icon?: string;
  parent_id?: string;
  sort_order?: number;
}

export interface UpdateGroupInput {
  id: string;
  name: string;
  icon?: string;
  sort_order?: number;
}

export type VaultError =
  | 'InvalidPassword'
  | 'VaultLocked'
  | 'ItemNotFound'
  | 'GroupNotEmpty'
  | 'CryptoError'
  | 'DatabaseError'
  | 'Validation';

/** Idle-lock policy owned by the Rust backend (persisted in vault_config). */
export interface SecuritySettings {
  auto_lock_minutes: number;
  /** Days a trashed item is kept before the automatic purge; `0` disables it. */
  trash_retention_days: number;
}

/** Metadata for an encrypted per-item attachment (contents stay in the vault). */
export interface AttachmentMeta {
  id: string;
  item_id: string;
  filename: string;
  content_type?: string | null;
  size: number;
  created_at: number;
}

/** One local snapshot of the encrypted database. */
export interface BackupInfo {
  file: string;
  created_at: number;
  size: number;
}

/** Per-item extra field (value encrypted at rest). */
export interface CustomField {
  id: string;
  key: string;
  value: string;
}

/** Cross-group label. */
export interface Tag {
  id: string;
  name: string;
  color?: string | null;
  created_at: number;
}

/** One activity-log record (never contains secret values). */
export interface AuditEntry {
  id: number;
  event: string;
  item_id?: string | null;
  item_title?: string | null;
  detail?: string | null;
  created_at: number;
}

/** A decrypted attachment fetched for preview/save. */
export interface AttachmentData {
  filename: string;
  content_type?: string | null;
  data_base64: string;
}

/**
 * One encrypted value that could not be decrypted. The backend only sends
 * metadata: no plaintext and no ciphertext ever reaches the UI.
 */
export interface IntegrityFailure {
  item_id: string;
  item_title: string;
  item_type: string;
  /** account | api_key | ... | custom_field | secret_history | attachment */
  scope: string;
  field: string;
  reason: string;
}

/** Result of the vault integrity self-check (`check_vault_integrity`). */
export interface IntegrityReport {
  /** Number of non-NULL ciphertexts that were decryption attempts. */
  checked_fields: number;
  failures: IntegrityFailure[];
  /** `"ok"` or the joined messages of SQLite's `PRAGMA integrity_check`. */
  sqlite_integrity: string;
  foreign_key_violations: string[];
  checked_attachments: boolean;
  checked_at: number;
  duration_ms: number;
}