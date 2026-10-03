/**
 * 纯浏览器调试用的 Tauri mock 层。
 *
 * 原理：Tauri v2 的所有 JS API（@tauri-apps/api/core 的 invoke、各插件、
 * window API）最终都通过 window.__TAURI_INTERNALS__.invoke 调到 Rust 侧。
 * 这里在进入真实 Tauri webview 之前用同名的内存实现替换它，前端代码零改动。
 *
 * 行为对齐后端契约（src-tauri/src/commands/*）：
 *   - 主密码规则与后端一致：任意 ≥8 位字符即可解锁（mock 不做哈希校验）
 *   - 参数同时接受 camelCase / snake_case（与 Tauri 的参数名转换行为一致）
 *   - 校验规则、错误码、锁定语义尽量与 Rust 实现一致
 *
 * 仅在开发模式下由入口文件动态加载；生产构建会被静态剔除。
 * 入口通过 maybeInstallBrowserMock() 判定环境：普通浏览器、或承载在
 * 其他 Tauri 应用壳里时安装 mock；SecKeyBox 自己的 webview 不安装。
 */
import type {
  Group,
  ItemSummary,
  ItemDetail,
  EnvVarPair,
} from '@/types';

type ItemType = 'account' | 'api_key' | 'env_var' | 'database' | 'ssh' | 'cloud' | 'license' | 'smtp';

type MockItem = {
  id: string;
  group_id: string;
  type: ItemType;
  title: string;
  icon?: string;
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  // account
  username?: string;
  password?: string;
  website?: string;
  // api_key
  key_name?: string;
  key_value?: string;
  endpoint?: string;
  auth_method?: string;
  rotation_date?: number;
  // env_var
  variables?: EnvVarPair[];
  // database
  db_type?: string;
  host?: string;
  port?: number;
  database_name?: string;
  connection_url?: string;
  // ssh
  key_path?: string;
  passphrase?: string;
  // cloud
  provider?: string;
  access_key_id?: string;
  region?: string;
  // license
  software_name?: string;
  license_key?: string;
  bound_email?: string;
  expiry_date?: number;
  // smtp
  encryption?: string;
  from_address?: string;
  // common
  notes?: string;
};

type MockState = {
  initialized: boolean;
  unlocked: boolean;
  groups: Group[];
  items: MockItem[];
  mockFs: Map<string, string>;
  lastExportPath: string | null;
};

const state: MockState = {
  initialized: true,
  unlocked: false,
  groups: [],
  items: [],
  mockFs: new Map(),
  lastExportPath: null,
};

// ---------- utils ----------

const now = () => Math.floor(Date.now() / 1000);

let seq = 0;
const uid = (prefix: string) =>
  `${prefix}-${Date.now().toString(36)}-${(seq++).toString(36)}`;

function vaultErr(code: string, message: string): Error {
  const e = new Error(`[${code}] ${message}`);
  e.name = code;
  return e;
}

function str(args: Record<string, unknown>, ...keys: string[]): string {
  for (const k of keys) {
    const v = args[k];
    if (typeof v === 'string') return v;
  }
  return '';
}

function optStr(args: Record<string, unknown>, ...keys: string[]): string | undefined {
  for (const k of keys) {
    const v = args[k];
    if (typeof v === 'string') return v;
  }
  return undefined;
}

function num(args: Record<string, unknown>, ...keys: string[]): number | undefined {
  for (const k of keys) {
    const v = args[k];
    if (typeof v === 'number') return v;
  }
  return undefined;
}

function pairs(args: Record<string, unknown>, key: string): EnvVarPair[] {
  const v = args[key];
  return Array.isArray(v) ? (v as EnvVarPair[]) : [];
}

const b64encode = (s: string): string => {
  const bytes = new TextEncoder().encode(s);
  let bin = '';
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  return btoa(bin);
};

const b64decode = (s: string): string => {
  try {
    const bin = atob(s);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return new TextDecoder().decode(bytes);
  } catch {
    return s; // 解不开就按明文处理，方便手工构造导入文件调试
  }
};

// ---------- 契约校验（对齐 items.rs / vault.rs） ----------

function requireUnlocked(): void {
  if (!state.unlocked) throw vaultErr('VaultLocked', 'Vault is locked');
}

function assertMasterPassword(pw: string): void {
  if (pw.length < 8) throw vaultErr('InvalidPassword', 'Password must be at least 8 characters');
}

function checkLength(value: string, min: number, max: number, label: string): void {
  if (value.length < min || value.length > max) {
    throw vaultErr('DatabaseError', `${label} must be ${min}-${max} characters`);
  }
}

// ---------- 内存数据 ----------

function findGroup(id: string): Group {
  const g = state.groups.find((x) => x.id === id);
  if (!g) throw vaultErr('ItemNotFound', `Group not found: ${id}`);
  return g;
}

function findItem(id: string): MockItem {
  const i = state.items.find((x) => x.id === id);
  if (!i) throw vaultErr('ItemNotFound', `Item not found: ${id}`);
  return i;
}

function baseItem(type: ItemType, groupId: string, title: string, icon?: string): MockItem {
  const t = now();
  return {
    id: uid('item'),
    group_id: groupId,
    type,
    title,
    icon,
    is_favorite: false,
    created_at: t,
    updated_at: t,
  };
}

function subtitleOf(item: MockItem): string {
  switch (item.type) {
    case 'account':
      return item.username ?? '';
    case 'api_key':
      return item.key_name ?? '';
    case 'env_var':
      return `${item.variables?.length ?? 0} variables`;
    case 'database':
      return `${item.host ?? ''}${item.port ? `:${item.port}` : ''}`;
    case 'ssh':
      return `${item.username ?? ''}@${item.host ?? ''}`;
    case 'cloud':
      return `${item.provider ?? ''} · ${item.access_key_id ?? ''}`;
    case 'license':
      return item.bound_email ?? '';
    case 'smtp':
      return `${item.host ?? ''}${item.port ? `:${item.port}` : ''}`;
  }
}

function toSummary(item: MockItem): ItemSummary {
  return {
    id: item.id,
    title: item.title,
    subtitle: subtitleOf(item),
    icon: item.icon,
    type: item.type,
    is_favorite: item.is_favorite,
    group_id: item.group_id,
    created_at: item.created_at,
    updated_at: item.updated_at,
  };
}

function toDetail(item: MockItem): ItemDetail {
  const common = {
    id: item.id,
    group_id: item.group_id,
    title: item.title,
    icon: item.icon,
    is_favorite: item.is_favorite,
    created_at: item.created_at,
    updated_at: item.updated_at,
  };
  switch (item.type) {
    case 'account':
      return {
        ...common,
        type: 'account',
        username: item.username ?? '',
        password: item.password ?? '',
        website: item.website,
        notes: item.notes,
      };
    case 'api_key':
      return {
        ...common,
        type: 'api_key',
        key_name: item.key_name ?? '',
        key_value: item.key_value ?? '',
        endpoint: item.endpoint,
        auth_method: item.auth_method,
        rotation_date: item.rotation_date,
        notes: item.notes,
      };
    case 'env_var':
      return {
        ...common,
        type: 'env_var',
        variables: item.variables ?? [],
        notes: item.notes,
      };
    case 'database':
      return {
        ...common,
        type: 'database',
        db_type: item.db_type ?? '',
        host: item.host ?? '',
        port: item.port,
        database_name: item.database_name,
        username: item.username,
        password: item.password,
        connection_url: item.connection_url,
        notes: item.notes,
      };
    case 'ssh':
      return {
        ...common,
        type: 'ssh',
        host: item.host ?? '',
        port: item.port,
        username: item.username ?? '',
        password: item.password,
        key_path: item.key_path,
        passphrase: item.passphrase,
        notes: item.notes,
      };
    case 'cloud':
      return {
        ...common,
        type: 'cloud',
        provider: item.provider ?? '',
        access_key_id: item.access_key_id ?? '',
        secret: item.password ?? '',
        region: item.region,
        notes: item.notes,
      };
    case 'license':
      return {
        ...common,
        type: 'license',
        software_name: item.software_name ?? '',
        license_key: item.license_key ?? '',
        bound_email: item.bound_email,
        expiry_date: item.expiry_date,
        notes: item.notes,
      };
    case 'smtp':
      return {
        ...common,
        type: 'smtp',
        host: item.host ?? '',
        port: item.port,
        encryption: item.encryption,
        username: item.username,
        password: item.password ?? '',
        from_address: item.from_address,
        notes: item.notes,
      };
  }
}

function seedDemoData(): void {
  const t = now();
  state.groups = [
    { id: 'built-in-accounts', name: 'Accounts', icon: '🔑', parent_id: undefined, sort_order: 0, created_at: t, updated_at: t },
    { id: 'built-in-api-keys', name: 'API Keys', icon: '🔧', parent_id: undefined, sort_order: 1, created_at: t, updated_at: t },
    { id: 'built-in-env-vars', name: 'Environment Variables', icon: '📦', parent_id: undefined, sort_order: 2, created_at: t, updated_at: t },
    { id: 'built-in-databases', name: 'Databases', icon: '🗄️', parent_id: undefined, sort_order: 3, created_at: t, updated_at: t },
    { id: 'built-in-servers', name: 'SSH Servers', icon: '🖥️', parent_id: undefined, sort_order: 4, created_at: t, updated_at: t },
    { id: 'built-in-cloud', name: 'Cloud Credentials', icon: '☁️', parent_id: undefined, sort_order: 5, created_at: t, updated_at: t },
    { id: 'built-in-licenses', name: 'Licenses', icon: '📜', parent_id: undefined, sort_order: 6, created_at: t, updated_at: t },
    { id: 'built-in-smtp', name: 'Email (SMTP)', icon: '✉️', parent_id: undefined, sort_order: 7, created_at: t, updated_at: t },
    { id: uid('group'), name: 'Work', icon: '🏢', parent_id: undefined, sort_order: 8, created_at: t, updated_at: t },
  ];
  const work = state.groups[8];
  state.items = [
    {
      ...baseItem('account', 'built-in-accounts', 'GitHub', '🐙'),
      username: 'octocat',
      password: 'gh-pass-12345',
      website: 'https://github.com',
      notes: '主账号，已开启 2FA',
      is_favorite: true,
    },
    {
      ...baseItem('account', work.id, 'GitLab (company)'),
      username: 'ethan@gitlab.example.com',
      password: 'gl-pass-67890',
      website: 'https://gitlab.example.com',
    },
    {
      ...baseItem('api_key', 'built-in-api-keys', 'OpenAI', '🤖'),
      key_name: 'OPENAI_API_KEY',
      key_value: 'sk-proj-abc123def456ghi789jkl012mno345',
      endpoint: 'https://api.openai.com/v1',
      auth_method: 'bearer',
      rotation_date: t + 60 * 86400,
    },
    {
      ...baseItem('env_var', 'built-in-env-vars', 'Prod Env'),
      variables: [
        { key: 'DATABASE_URL', value: 'postgres://app:s3cr3t@db.prod:5432/appdb' },
        { key: 'REDIS_URL', value: 'redis://cache.prod:6379/0' },
        { key: 'API_SECRET', value: 'env-secret-do-not-share' },
      ],
      notes: '生产环境变量，轮换前勿外发',
    },
    {
      ...baseItem('database', 'built-in-databases', 'App 主库'),
      db_type: 'postgresql',
      host: 'db.prod.internal',
      port: 5432,
      database_name: 'appdb',
      username: 'app',
      password: 'db-pass-9527',
      connection_url: 'postgres://app:db-pass-9527@db.prod.internal:5432/appdb',
      notes: '生产主库，只读账号另存',
    },
    {
      ...baseItem('ssh', 'built-in-servers', '跳板机'),
      host: 'bastion.prod.internal',
      port: 22,
      username: 'ethan',
      password: 'ssh-pass-3344',
      key_path: '~/.ssh/id_ed25519',
      passphrase: 'key-pass-5566',
    },
    {
      ...baseItem('cloud', 'built-in-cloud', 'AWS 主账号'),
      provider: 'aws',
      access_key_id: 'AKIAIOSFODNN7EXAMPLE',
      password: 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY',
      region: 'us-east-1',
      notes: 'root access key，建议换 IAM',
    },
    {
      ...baseItem('license', 'built-in-licenses', 'JetBrains 全家桶'),
      software_name: 'JetBrains All Products Pack',
      license_key: 'LICENSE-KEY-ABCD-1234-EFGH',
      bound_email: 'ethan@example.com',
      expiry_date: t + 365 * 86400,
    },
    {
      ...baseItem('smtp', 'built-in-smtp', '通知邮件'),
      host: 'smtp.example.com',
      port: 587,
      encryption: 'starttls',
      username: 'noreply@example.com',
      password: 'smtp-pass-7788',
      from_address: 'noreply@example.com',
    },
  ];
}

// ---------- 导出 / 导入（对齐 import_export.rs 的 JSON 结构） ----------

function randomNonceB64(): string {
  const bytes = new Uint8Array(12);
  crypto.getRandomValues(bytes);
  let bin = '';
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  return btoa(bin);
}

function buildExport(): unknown {
  return {
    version: '1.0',
    exported_at: now(),
    app: 'SecKeyBox',
    groups: [...state.groups].sort((a, b) => a.sort_order - b.sort_order),
    items: state.items.map((item) => {
      let encrypted: Record<string, unknown>;
      switch (item.type) {
        case 'account':
          encrypted = {
            username: item.username ?? '',
            password_encrypted: b64encode(item.password ?? ''),
            password_nonce: randomNonceB64(),
            website: item.website ?? null,
            notes: item.notes ?? null,
          };
          break;
        case 'api_key':
          encrypted = {
            key_name: item.key_name ?? '',
            key_value_encrypted: b64encode(item.key_value ?? ''),
            key_value_nonce: randomNonceB64(),
            endpoint: item.endpoint ?? null,
            auth_method: item.auth_method ?? null,
            notes: item.notes ?? null,
            rotation_date: item.rotation_date ?? null,
          };
          break;
        case 'env_var':
          encrypted = {
            variables: (item.variables ?? []).map((v) => ({
              key: v.key,
              value_encrypted: b64encode(v.value),
              value_nonce: randomNonceB64(),
            })),
            notes: item.notes ?? null,
          };
          break;
        case 'database':
          encrypted = {
            db_type: item.db_type ?? '',
            host: item.host ?? '',
            port: item.port ?? null,
            database_name: item.database_name ?? null,
            username: item.username ?? null,
            password_encrypted: item.password != null ? b64encode(item.password) : null,
            password_nonce: item.password != null ? randomNonceB64() : null,
            connection_url: item.connection_url ?? null,
            notes: item.notes ?? null,
          };
          break;
        case 'ssh':
          encrypted = {
            host: item.host ?? '',
            port: item.port ?? null,
            username: item.username ?? '',
            password_encrypted: item.password != null ? b64encode(item.password) : null,
            password_nonce: item.password != null ? randomNonceB64() : null,
            key_path: item.key_path ?? null,
            passphrase_encrypted: item.passphrase != null ? b64encode(item.passphrase) : null,
            passphrase_nonce: item.passphrase != null ? randomNonceB64() : null,
            notes: item.notes ?? null,
          };
          break;
        case 'cloud':
          encrypted = {
            provider: item.provider ?? '',
            access_key_id: item.access_key_id ?? '',
            secret_encrypted: b64encode(item.password ?? ''),
            secret_nonce: randomNonceB64(),
            region: item.region ?? null,
            notes: item.notes ?? null,
          };
          break;
        case 'license':
          encrypted = {
            software_name: item.software_name ?? '',
            license_key_encrypted: b64encode(item.license_key ?? ''),
            license_key_nonce: randomNonceB64(),
            bound_email: item.bound_email ?? null,
            expiry_date: item.expiry_date ?? null,
            notes: item.notes ?? null,
          };
          break;
        case 'smtp':
          encrypted = {
            host: item.host ?? '',
            port: item.port ?? null,
            encryption: item.encryption ?? null,
            username: item.username ?? null,
            password_encrypted: b64encode(item.password ?? ''),
            password_nonce: randomNonceB64(),
            from_address: item.from_address ?? null,
            notes: item.notes ?? null,
          };
          break;
      }
      return {
        id: item.id,
        group_id: item.group_id,
        title: item.title,
        icon: item.icon ?? null,
        type: item.type,
        is_favorite: item.is_favorite,
        created_at: item.created_at,
        updated_at: item.updated_at,
        encrypted_data: encrypted,
      };
    }),
  };
}

type ExportItemLike = {
  id: string;
  group_id: string;
  title: string;
  icon?: string | null;
  type: string;
  is_favorite?: boolean;
  created_at?: number;
  updated_at?: number;
  encrypted_data?: Record<string, unknown>;
};

function importFromExport(data: string, mode: string): Record<string, number> {
  let parsed: { app?: string; groups?: Group[]; items?: ExportItemLike[] };
  try {
    parsed = JSON.parse(data);
  } catch (e) {
    throw vaultErr('DatabaseError', `Invalid export format: ${String(e)}`);
  }
  if (parsed.app !== 'SecKeyBox') {
    throw vaultErr('DatabaseError', 'Not a SecKeyBox export file');
  }

  if (mode === 'replace') {
    state.items = [];
    state.groups = state.groups.filter((g) => g.id.startsWith('built-in-'));
  }

  const result = { groups_imported: 0, items_imported: 0, groups_skipped: 0, items_skipped: 0 };

  for (const g of parsed.groups ?? []) {
    if (state.groups.some((x) => x.id === g.id) && mode === 'merge') {
      result.groups_skipped++;
      continue;
    }
    const t = now();
    const existing = state.groups.find((x) => x.id === g.id);
    const merged: Group = {
      id: g.id,
      name: g.name,
      icon: g.icon ?? undefined,
      parent_id: g.parent_id ?? undefined,
      sort_order: g.sort_order ?? 999,
      created_at: g.created_at ?? t,
      updated_at: g.updated_at ?? t,
    };
    if (existing) Object.assign(existing, merged);
    else state.groups.push(merged);
    result.groups_imported++;
  }

  for (const it of parsed.items ?? []) {
    if (state.items.some((x) => x.id === it.id) && mode === 'merge') {
      result.items_skipped++;
      continue;
    }
    const type = it.type as ItemType;
    const knownTypes: ItemType[] = ['account', 'api_key', 'env_var', 'database', 'ssh', 'cloud', 'license', 'smtp'];
    if (!knownTypes.includes(type)) continue;
    const ed = it.encrypted_data ?? {};
    const item: MockItem = {
      id: it.id,
      group_id: it.group_id,
      type,
      title: it.title,
      icon: it.icon ?? undefined,
      is_favorite: Boolean(it.is_favorite),
      created_at: it.created_at ?? now(),
      updated_at: it.updated_at ?? now(),
    };
    switch (type) {
      case 'account':
        item.username = typeof ed.username === 'string' ? ed.username : '';
        item.password = b64decode(typeof ed.password_encrypted === 'string' ? ed.password_encrypted : '');
        item.website = typeof ed.website === 'string' ? ed.website : undefined;
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
      case 'api_key':
        item.key_name = typeof ed.key_name === 'string' ? ed.key_name : '';
        item.key_value = b64decode(typeof ed.key_value_encrypted === 'string' ? ed.key_value_encrypted : '');
        item.endpoint = typeof ed.endpoint === 'string' ? ed.endpoint : undefined;
        item.auth_method = typeof ed.auth_method === 'string' ? ed.auth_method : undefined;
        item.rotation_date = typeof ed.rotation_date === 'number' ? ed.rotation_date : undefined;
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
      case 'env_var': {
        const vars = Array.isArray(ed.variables) ? ed.variables : [];
        item.variables = vars.map((v) => ({
          key: String(v?.key ?? ''),
          value: b64decode(String(v?.value_encrypted ?? '')),
        }));
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
      }
      case 'database':
        item.db_type = typeof ed.db_type === 'string' ? ed.db_type : '';
        item.host = typeof ed.host === 'string' ? ed.host : '';
        item.port = typeof ed.port === 'number' ? ed.port : undefined;
        item.database_name = typeof ed.database_name === 'string' ? ed.database_name : undefined;
        item.username = typeof ed.username === 'string' ? ed.username : undefined;
        item.password =
          typeof ed.password_encrypted === 'string' && ed.password_encrypted
            ? b64decode(ed.password_encrypted)
            : undefined;
        item.connection_url = typeof ed.connection_url === 'string' ? ed.connection_url : undefined;
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
      case 'ssh':
        item.host = typeof ed.host === 'string' ? ed.host : '';
        item.port = typeof ed.port === 'number' ? ed.port : undefined;
        item.username = typeof ed.username === 'string' ? ed.username : '';
        item.password =
          typeof ed.password_encrypted === 'string' && ed.password_encrypted
            ? b64decode(ed.password_encrypted)
            : undefined;
        item.key_path = typeof ed.key_path === 'string' ? ed.key_path : undefined;
        item.passphrase =
          typeof ed.passphrase_encrypted === 'string' && ed.passphrase_encrypted
            ? b64decode(ed.passphrase_encrypted)
            : undefined;
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
      case 'cloud':
        item.provider = typeof ed.provider === 'string' ? ed.provider : '';
        item.access_key_id = typeof ed.access_key_id === 'string' ? ed.access_key_id : '';
        item.password = b64decode(typeof ed.secret_encrypted === 'string' ? ed.secret_encrypted : '');
        item.region = typeof ed.region === 'string' ? ed.region : undefined;
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
      case 'license':
        item.software_name = typeof ed.software_name === 'string' ? ed.software_name : '';
        item.license_key = b64decode(
          typeof ed.license_key_encrypted === 'string' ? ed.license_key_encrypted : '',
        );
        item.bound_email = typeof ed.bound_email === 'string' ? ed.bound_email : undefined;
        item.expiry_date = typeof ed.expiry_date === 'number' ? ed.expiry_date : undefined;
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
      case 'smtp':
        item.host = typeof ed.host === 'string' ? ed.host : '';
        item.port = typeof ed.port === 'number' ? ed.port : undefined;
        item.encryption = typeof ed.encryption === 'string' ? ed.encryption : undefined;
        item.username = typeof ed.username === 'string' ? ed.username : undefined;
        item.password = b64decode(typeof ed.password_encrypted === 'string' ? ed.password_encrypted : '');
        item.from_address = typeof ed.from_address === 'string' ? ed.from_address : undefined;
        item.notes = typeof ed.notes === 'string' ? ed.notes : undefined;
        break;
    }
    const existing = state.items.find((x) => x.id === it.id);
    if (existing) state.items.splice(state.items.indexOf(existing), 1, item);
    else state.items.push(item);
    result.items_imported++;
  }

  return result;
}

// ---------- 命令表（对齐 lib.rs generate_handler 的 25 个命令） ----------

const commands: Record<string, (args: Record<string, unknown>) => unknown> = {
  is_vault_initialized: () => state.initialized,

  initialize_vault: (a) => {
    const pw = str(a, 'masterPassword', 'master_password');
    assertMasterPassword(pw);
    if (state.initialized) throw vaultErr('VaultAlreadyInitialized', 'Vault already initialized');
    state.initialized = true;
    state.unlocked = true;
    return null;
  },

  unlock_vault: (a) => {
    assertMasterPassword(str(a, 'masterPassword', 'master_password'));
    state.unlocked = true;
    return true;
  },

  lock_vault: () => {
    state.unlocked = false;
    return null;
  },

  is_vault_unlocked: () => state.unlocked,

  get_groups: () => [...state.groups].sort((a, b) => a.sort_order - b.sort_order),

  create_new_group: (a) => {
    const group: Group = {
      id: uid('group'),
      name: str(a, 'name'),
      icon: optStr(a, 'icon'),
      parent_id: optStr(a, 'parentId', 'parent_id'),
      sort_order: num(a, 'sortOrder', 'sort_order') ?? 999,
      created_at: now(),
      updated_at: now(),
    };
    state.groups.push(group);
    return group;
  },

  update_existing_group: (a) => {
    const g = findGroup(str(a, 'id'));
    g.name = str(a, 'name');
    const icon = optStr(a, 'icon');
    if (icon !== undefined) g.icon = icon;
    const so = num(a, 'sortOrder', 'sort_order');
    if (so !== undefined) g.sort_order = so;
    g.updated_at = now();
    return null;
  },

  delete_existing_group: (a) => {
    const id = str(a, 'id');
    if (id.startsWith('built-in-')) {
      throw vaultErr('DatabaseError', 'Cannot delete built-in group');
    }
    state.groups = state.groups.filter((g) => g.id !== id);
    state.items = state.items.filter((i) => i.group_id !== id);
    return null;
  },

  get_all_items_cmd: () => state.items.map(toSummary),

  get_items_by_group_cmd: (a) => {
    const gid = str(a, 'group_id', 'groupId');
    return state.items.filter((i) => i.group_id === gid).map(toSummary);
  },

  get_item_detail: (a) => {
    requireUnlocked();
    return toDetail(findItem(str(a, 'id')));
  },

  create_new_account_item: (a) => {
    requireUnlocked();
    const title = str(a, 'title');
    const username = str(a, 'username');
    const password = str(a, 'password');
    checkLength(title, 1, 100, 'Title');
    checkLength(username, 1, 100, 'Username');
    checkLength(password, 1, 1000, 'Password');
    const item = baseItem('account', str(a, 'group_id', 'groupId'), title, optStr(a, 'icon'));
    item.username = username;
    item.password = password;
    item.website = optStr(a, 'website');
    item.notes = optStr(a, 'notes');
    state.items.push(item);
    return item.id;
  },

  update_existing_account_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'account') throw vaultErr('ItemNotFound', 'Not an account item');
    const title = str(a, 'title');
    const username = str(a, 'username');
    checkLength(title, 1, 100, 'Title');
    checkLength(username, 1, 100, 'Username');
    const password = optStr(a, 'password');
    if (password !== undefined) checkLength(password, 1, 1000, 'Password');
    item.title = title;
    item.username = username;
    if (password !== undefined) item.password = password;
    item.website = optStr(a, 'website');
    item.notes = optStr(a, 'notes');
    item.updated_at = now();
    return null;
  },

  create_new_api_key_item: (a) => {
    requireUnlocked();
    const r = (a.request ?? a) as Record<string, unknown>;
    const title = str(r, 'title');
    const keyName = str(r, 'key_name', 'keyName');
    const keyValue = str(r, 'key_value', 'keyValue');
    checkLength(title, 1, 100, 'Title');
    checkLength(keyName, 1, 100, 'Key name');
    checkLength(keyValue, 1, 50000, 'Key value');
    const endpoint = optStr(r, 'endpoint');
    if (endpoint !== undefined) checkLength(endpoint, 0, 500, 'Endpoint');
    const authMethod = optStr(r, 'auth_method', 'authMethod');
    if (authMethod !== undefined) checkLength(authMethod, 0, 20, 'Auth method');
    const notes = optStr(r, 'notes');
    if (notes !== undefined) checkLength(notes, 0, 5000, 'Notes');
    const item = baseItem(
      'api_key',
      str(r, 'group_id', 'groupId'),
      title,
      optStr(r, 'icon'),
    );
    item.key_name = keyName;
    item.key_value = keyValue;
    item.endpoint = endpoint;
    item.auth_method = authMethod;
    item.notes = notes;
    state.items.push(item);
    return item.id;
  },

  // 真实后端签名为平铺参数 (id, title, key_name, ...)，不是 request 包装。
  // stores/vault.ts 目前用 { request: {...} } 调用本命令 —— 与后端不符，
  // mock 刻意保持与后端一致，让这个问题在浏览器调试时同样暴露。
  update_existing_api_key_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'api_key') throw vaultErr('ItemNotFound', 'Not an api_key item');
    const title = str(a, 'title');
    const keyName = str(a, 'key_name', 'keyName');
    checkLength(title, 1, 100, 'Title');
    checkLength(keyName, 1, 100, 'Key name');
    const keyValue = optStr(a, 'key_value', 'keyValue');
    if (keyValue !== undefined) checkLength(keyValue, 1, 50000, 'Key value');
    const endpoint = optStr(a, 'endpoint');
    if (endpoint !== undefined) checkLength(endpoint, 0, 500, 'Endpoint');
    const authMethod = optStr(a, 'auth_method', 'authMethod');
    if (authMethod !== undefined) checkLength(authMethod, 0, 20, 'Auth method');
    const notes = optStr(a, 'notes');
    if (notes !== undefined) checkLength(notes, 0, 5000, 'Notes');
    item.title = title;
    item.key_name = keyName;
    if (keyValue !== undefined) item.key_value = keyValue;
    item.endpoint = endpoint;
    item.auth_method = authMethod;
    item.notes = notes;
    item.updated_at = now();
    return null;
  },

  create_new_env_var_item: (a) => {
    requireUnlocked();
    const title = str(a, 'title');
    const variables = pairs(a, 'variables');
    checkLength(title, 1, 100, 'Title');
    if (variables.length === 0) {
      throw vaultErr('DatabaseError', 'At least one variable is required');
    }
    for (const v of variables) {
      checkLength(v.key, 1, 100, 'Variable key');
      checkLength(v.value, 1, 5000, 'Variable value');
    }
    const notes = optStr(a, 'notes');
    if (notes !== undefined) checkLength(notes, 0, 5000, 'Notes');
    const item = baseItem('env_var', str(a, 'group_id', 'groupId'), title, optStr(a, 'icon'));
    item.variables = variables;
    item.notes = notes;
    state.items.push(item);
    return item.id;
  },

  update_existing_env_var_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'env_var') throw vaultErr('ItemNotFound', 'Not an env_var item');
    const title = str(a, 'title');
    const variables = pairs(a, 'variables');
    checkLength(title, 1, 100, 'Title');
    if (variables.length === 0) {
      throw vaultErr('DatabaseError', 'At least one variable is required');
    }
    item.title = title;
    item.variables = variables;
    item.notes = optStr(a, 'notes');
    item.updated_at = now();
    return null;
  },

  create_new_database_item: (a) => {
    requireUnlocked();
    const title = str(a, 'title');
    const host = str(a, 'host');
    checkLength(title, 1, 100, 'Title');
    checkLength(str(a, 'dbType', 'db_type'), 1, 20, 'Database type');
    checkLength(host, 1, 255, 'Host');
    const item = baseItem('database', str(a, 'group_id', 'groupId'), title, optStr(a, 'icon'));
    item.db_type = str(a, 'dbType', 'db_type');
    item.host = host;
    item.port = num(a, 'port');
    item.database_name = optStr(a, 'databaseName', 'database_name');
    item.username = optStr(a, 'username');
    item.password = optStr(a, 'password');
    item.connection_url = optStr(a, 'connectionUrl', 'connection_url');
    item.notes = optStr(a, 'notes');
    state.items.push(item);
    return item.id;
  },

  update_existing_database_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'database') throw vaultErr('ItemNotFound', 'Not a database item');
    const title = str(a, 'title');
    const host = str(a, 'host');
    checkLength(title, 1, 100, 'Title');
    checkLength(host, 1, 255, 'Host');
    const password = optStr(a, 'password');
    item.title = title;
    item.db_type = str(a, 'dbType', 'db_type');
    item.host = host;
    item.port = num(a, 'port');
    item.database_name = optStr(a, 'databaseName', 'database_name');
    item.username = optStr(a, 'username');
    if (password !== undefined) item.password = password;
    item.connection_url = optStr(a, 'connectionUrl', 'connection_url');
    item.notes = optStr(a, 'notes');
    item.updated_at = now();
    return null;
  },

  create_new_ssh_item: (a) => {
    requireUnlocked();
    const title = str(a, 'title');
    const host = str(a, 'host');
    const username = str(a, 'username');
    checkLength(title, 1, 100, 'Title');
    checkLength(host, 1, 255, 'Host');
    checkLength(username, 1, 100, 'Username');
    const item = baseItem('ssh', str(a, 'group_id', 'groupId'), title, optStr(a, 'icon'));
    item.host = host;
    item.port = num(a, 'port');
    item.username = username;
    item.password = optStr(a, 'password');
    item.key_path = optStr(a, 'keyPath', 'key_path');
    item.passphrase = optStr(a, 'passphrase');
    item.notes = optStr(a, 'notes');
    state.items.push(item);
    return item.id;
  },

  update_existing_ssh_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'ssh') throw vaultErr('ItemNotFound', 'Not an ssh item');
    const title = str(a, 'title');
    const host = str(a, 'host');
    const username = str(a, 'username');
    checkLength(title, 1, 100, 'Title');
    checkLength(host, 1, 255, 'Host');
    checkLength(username, 1, 100, 'Username');
    const password = optStr(a, 'password');
    const passphrase = optStr(a, 'passphrase');
    item.title = title;
    item.host = host;
    item.port = num(a, 'port');
    item.username = username;
    if (password !== undefined) item.password = password;
    item.key_path = optStr(a, 'keyPath', 'key_path');
    if (passphrase !== undefined) item.passphrase = passphrase;
    item.notes = optStr(a, 'notes');
    item.updated_at = now();
    return null;
  },

  create_new_cloud_item: (a) => {
    requireUnlocked();
    const title = str(a, 'title');
    const provider = str(a, 'provider');
    const accessKeyId = str(a, 'accessKeyId', 'access_key_id');
    const secret = str(a, 'secret');
    checkLength(title, 1, 100, 'Title');
    checkLength(provider, 1, 30, 'Provider');
    checkLength(accessKeyId, 1, 200, 'Access Key ID');
    checkLength(secret, 1, 1000, 'Secret');
    const item = baseItem('cloud', str(a, 'group_id', 'groupId'), title, optStr(a, 'icon'));
    item.provider = provider;
    item.access_key_id = accessKeyId;
    item.password = secret; // mock 里 cloud 的 secret 复用 password 字段
    item.region = optStr(a, 'region');
    item.notes = optStr(a, 'notes');
    state.items.push(item);
    return item.id;
  },

  update_existing_cloud_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'cloud') throw vaultErr('ItemNotFound', 'Not a cloud item');
    const title = str(a, 'title');
    checkLength(title, 1, 100, 'Title');
    checkLength(str(a, 'provider'), 1, 30, 'Provider');
    checkLength(str(a, 'accessKeyId', 'access_key_id'), 1, 200, 'Access Key ID');
    const secret = optStr(a, 'secret');
    item.title = title;
    item.provider = str(a, 'provider');
    item.access_key_id = str(a, 'accessKeyId', 'access_key_id');
    if (secret !== undefined) item.password = secret;
    item.region = optStr(a, 'region');
    item.notes = optStr(a, 'notes');
    item.updated_at = now();
    return null;
  },

  create_new_license_item: (a) => {
    requireUnlocked();
    const title = str(a, 'title');
    const softwareName = str(a, 'softwareName', 'software_name');
    const licenseKey = str(a, 'licenseKey', 'license_key');
    checkLength(title, 1, 100, 'Title');
    checkLength(softwareName, 1, 100, 'Software name');
    checkLength(licenseKey, 1, 1000, 'License key');
    const item = baseItem('license', str(a, 'group_id', 'groupId'), title, optStr(a, 'icon'));
    item.software_name = softwareName;
    item.license_key = licenseKey;
    item.bound_email = optStr(a, 'boundEmail', 'bound_email');
    item.expiry_date = num(a, 'expiryDate', 'expiry_date');
    item.notes = optStr(a, 'notes');
    state.items.push(item);
    return item.id;
  },

  update_existing_license_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'license') throw vaultErr('ItemNotFound', 'Not a license item');
    const title = str(a, 'title');
    checkLength(title, 1, 100, 'Title');
    checkLength(str(a, 'softwareName', 'software_name'), 1, 100, 'Software name');
    const licenseKey = optStr(a, 'licenseKey', 'license_key');
    item.title = title;
    item.software_name = str(a, 'softwareName', 'software_name');
    if (licenseKey !== undefined) item.license_key = licenseKey;
    item.bound_email = optStr(a, 'boundEmail', 'bound_email');
    item.expiry_date = num(a, 'expiryDate', 'expiry_date');
    item.notes = optStr(a, 'notes');
    item.updated_at = now();
    return null;
  },

  create_new_smtp_item: (a) => {
    requireUnlocked();
    const title = str(a, 'title');
    const host = str(a, 'host');
    const password = str(a, 'password');
    checkLength(title, 1, 100, 'Title');
    checkLength(host, 1, 255, 'Host');
    checkLength(password, 1, 1000, 'Password');
    const item = baseItem('smtp', str(a, 'group_id', 'groupId'), title, optStr(a, 'icon'));
    item.host = host;
    item.port = num(a, 'port');
    item.encryption = optStr(a, 'encryption');
    item.username = optStr(a, 'username');
    item.password = password;
    item.from_address = optStr(a, 'fromAddress', 'from_address');
    item.notes = optStr(a, 'notes');
    state.items.push(item);
    return item.id;
  },

  update_existing_smtp_item: (a) => {
    requireUnlocked();
    const item = findItem(str(a, 'id'));
    if (item.type !== 'smtp') throw vaultErr('ItemNotFound', 'Not an smtp item');
    const title = str(a, 'title');
    const host = str(a, 'host');
    checkLength(title, 1, 100, 'Title');
    checkLength(host, 1, 255, 'Host');
    const password = optStr(a, 'password');
    item.title = title;
    item.host = host;
    item.port = num(a, 'port');
    item.encryption = optStr(a, 'encryption');
    item.username = optStr(a, 'username');
    if (password !== undefined) item.password = password;
    item.from_address = optStr(a, 'fromAddress', 'from_address');
    item.notes = optStr(a, 'notes');
    item.updated_at = now();
    return null;
  },

  delete_existing_item: (a) => {
    const id = str(a, 'id');
    state.items = state.items.filter((i) => i.id !== id);
    return null;
  },

  toggle_item_favorite: (a) => {
    const item = findItem(str(a, 'id'));
    item.is_favorite = !item.is_favorite;
    item.updated_at = now();
    return null;
  },

  copy_to_clipboard: async (a) => {
    const text = str(a, 'text');
    try {
      await navigator.clipboard.writeText(text);
      console.info('[mock clipboard] write:', text.length > 40 ? `${text.slice(0, 40)}...` : text);
    } catch {
      console.warn('[mock clipboard] navigator.clipboard 不可用，写入被忽略');
    }
    return null;
  },

  clear_clipboard: async () => {
    try {
      await navigator.clipboard.writeText('');
    } catch {
      /* ignore */
    }
    return null;
  },

  search_items: (a) => {
    if (!state.unlocked) return [];
    const q = str(a, 'query').toLowerCase();
    const all = state.items.map(toSummary);
    if (!q) return all;
    return all.filter(
      (i) => i.title.toLowerCase().includes(q) || i.subtitle.toLowerCase().includes(q),
    );
  },

  export_vault: () => {
    requireUnlocked();
    return JSON.stringify(buildExport(), null, 2);
  },

  import_vault: (a) => {
    requireUnlocked();
    return importFromExport(str(a, 'data'), str(a, 'mode') || 'merge');
  },
};

// ---------- 插件命令（dialog / fs / window） ----------

function triggerBrowserDownload(filename: string, content: string): void {
  try {
    const blob = new Blob([content], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  } catch {
    /* ignore */
  }
}

type PluginHandler = (
  args: Record<string, unknown>,
  options?: { headers?: Record<string, unknown> } | null,
) => unknown;

// 真实 plugin-fs 的通道约定：
//   writeTextFile → invoke('plugin:fs|write_text_file', TextEncoder(body), { headers: { path: encodeURIComponent(p) } })
//   readTextFile  → invoke('plugin:fs|read_text_file', { path }, ...) → 返回 ArrayBuffer/byte 数组
const pluginHandlers: Record<string, PluginHandler> = {
  'plugin:dialog|open': (a) => {
    const opts = (a.options ?? a) as Record<string, unknown>;
    // 导出流程带 defaultPath：模拟用户在对话框中确认保存位置
    if (typeof opts.defaultPath === 'string' && opts.defaultPath) {
      const p = `/mock/${opts.defaultPath}`;
      console.info('[mock dialog] open(save) ->', p);
      return p;
    }
    // 导入流程：返回 mock 文件系统中最新的导出文件，没有则模拟取消
    if (state.mockFs.size === 0) {
      console.info('[mock dialog] open: mock 文件系统为空，返回 null（先 Export 一次即可获得可导入文件）');
      return null;
    }
    const p = state.lastExportPath ?? [...state.mockFs.keys()][0];
    console.info('[mock dialog] open ->', p);
    return p;
  },

  'plugin:fs|write_text_file': (body, options) => {
    const headerPath = options?.headers?.path;
    const path =
      typeof headerPath === 'string'
        ? decodeURIComponent(headerPath)
        : str(body, 'path'); // 兼容旧版插件的 {path, contents} 形态
    let contents = '';
    if (body instanceof Uint8Array || body instanceof ArrayBuffer) {
      contents = new TextDecoder().decode(body);
    } else if (typeof body === 'string') {
      contents = body;
    } else {
      contents = str(body, 'contents');
    }
    state.mockFs.set(path, contents);
    state.lastExportPath = path;
    triggerBrowserDownload(path.split('/').pop() ?? 'export.json', contents);
    console.info(`[mock fs] wrote ${path}（${contents.length} 字符，同时已触发浏览器下载）`);
    return null;
  },

  'plugin:fs|read_text_file': (a) => {
    const path = str(a, 'path');
    const content = state.mockFs.get(path);
    if (content === undefined) {
      throw vaultErr('FsError', `mock fs: file not found: ${path}`);
    }
    // 真实插件返回字节序列，插件 JS 层再 decode —— 这里保持一致
    return new TextEncoder().encode(content);
  },
};

// ---------- 安装 ----------

/** 尽力把字段补到宿主对象上；普通赋值失败（属性被保护）时退回 defineProperty */
function patchProp(obj: Record<string, unknown>, key: string, value: unknown): boolean {
  try {
    obj[key] = value;
    if (obj[key] === value) return true;
  } catch {
    /* fallthrough */
  }
  try {
    Object.defineProperty(obj, key, { value, configurable: true, writable: true });
    return obj[key] === value;
  } catch {
    return false;
  }
}

export function installTauriBrowserMock(): void {
  const w = window as unknown as Record<string, unknown>;
  if ('__SECKEYBOX_MOCK__' in w) return; // 已安装，幂等

  seedDemoData();

  const currentLabel = 'main';

  const shim = {
    metadata: {
      currentWindow: { label: currentLabel },
      currentWebview: { label: currentLabel },
    },
    transformCallback: (callback: (event?: unknown) => void, once = false): number => {
      const id = ++seq;
      const key = `_${id}`;
      (w as Record<string, unknown>)[key] = (event?: unknown) => {
        if (once) delete (w as Record<string, unknown>)[key];
        callback(event);
      };
      return id;
    },
    invoke: async (
      cmd: string,
      args: Record<string, unknown> = {},
      options?: { headers?: Record<string, unknown> } | null,
    ): Promise<unknown> => {
      if (cmd.startsWith('plugin:')) {
        const handler = pluginHandlers[cmd];
        if (!handler) {
          console.warn(`[SecKeyBox mock] 未实现的插件命令（返回 null）: ${cmd}`, args);
          return null;
        }
        return handler(args, options);
      }
      const handler = commands[cmd];
      if (!handler) {
        throw vaultErr('CommandNotFound', `command not found: ${cmd}`);
      }
      return handler(args);
    },
  };

  // 宿主壳（如内嵌 WebView）可能已注入受保护的 __TAURI_INTERNALS__：
  // 此时不能整对象覆盖（会抛错），只在其上补齐缺失的 invoke/metadata。
  const existing = w.__TAURI_INTERNALS__ as Record<string, unknown> | undefined;
  if (existing) {
    if (!patchProp(existing, 'invoke', shim.invoke)) {
      w.__SECKEYBOX_MOCK__ = { failed: true };
      console.error('[SecKeyBox mock] 无法接管宿主 __TAURI_INTERNALS__.invoke，mock 未启用');
      return;
    }
    patchProp(existing, 'metadata', shim.metadata);
    patchProp(existing, 'transformCallback', shim.transformCallback);
  } else {
    w.__TAURI_INTERNALS__ = shim;
  }

  w.__SECKEYBOX_MOCK__ = {
    /** 重置为初始演示数据并回到锁定态 */
    reset: () => {
      state.initialized = true;
      state.unlocked = false;
      state.mockFs.clear();
      state.lastExportPath = null;
      seedDemoData();
      console.info('[SecKeyBox mock] 已重置（刷新页面生效）');
    },
    /** 回到首次安装流程（SetupScreen） */
    setInitialized: (v: boolean) => {
      state.initialized = v;
      state.unlocked = v;
      console.info(`[SecKeyBox mock] initialized=${v}（刷新页面生效）`);
    },
    /** 查看当前内存数据 */
    dump: () => ({ ...state, mockFs: [...state.mockFs.keys()] }),
  };

  console.info(
    '[SecKeyBox mock] 浏览器 mock 已启用：任意 ≥8 位密码即可解锁；' +
      'window.__SECKEYBOX_MOCK__.reset() 可重置演示数据',
  );
}

/**
 * 判断是否需要安装浏览器 mock，需要则安装。
 *
 * 三种环境的判定：
 *   1. 无 __TAURI_INTERNALS__（普通浏览器标签页）→ 安装 mock
 *   2. 有 internals，且 invoke('is_vault_initialized') 能正常返回
 *      （SecKeyBox 自己的 webview，如 npm run tauri dev）→ 不安装
 *   3. 有 internals，但探测失败/超时（其他 Tauri 应用的 webview，
 *      例如把 dev 页面嵌进别的 Tauri 壳里打开）→ 安装 mock
 */
export async function maybeInstallBrowserMock(): Promise<boolean> {
  const w = window as unknown as {
    __TAURI_INTERNALS__?: { invoke: (cmd: string) => Promise<unknown> };
  };
  const internals = w.__TAURI_INTERNALS__;
  if (!internals) {
    installTauriBrowserMock();
    return true;
  }
  try {
    await Promise.race([
      internals.invoke('is_vault_initialized'),
      new Promise((_resolve, reject) => setTimeout(() => reject(new Error('probe timeout')), 800)),
    ]);
    return false; // 真实 SecKeyBox 后端可用，保持原样
  } catch {
    installTauriBrowserMock();
    return true;
  }
}
