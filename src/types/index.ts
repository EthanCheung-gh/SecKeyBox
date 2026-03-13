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
  type: 'account' | 'api_key' | 'env_var';
  is_favorite: boolean;
  group_id: string;
  created_at: number;
  updated_at: number;
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

export type ItemDetail = AccountItemDetail | ApiKeyItemDetail | EnvVarItemDetail;

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
  | 'DatabaseError';