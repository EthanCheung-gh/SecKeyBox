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
  type: 'account';
  is_favorite: boolean;
  group_id: string;
  created_at: number;
  updated_at: number;
}

export interface ItemDetail {
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