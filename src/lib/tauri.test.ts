import { beforeEach, describe, expect, it, vi } from 'vitest';

/**
 * 回归测试：Tauri v2 命令参数命名。
 *
 * Tauri v2 会把 JS 侧的参数名映射到 Rust 的 snake_case 形参，因此调用侧
 * 必须使用 camelCase（`groupId`），否则报
 * `invalid args groupId for command ...: missing required key groupId`。
 * 唯一例外是整体嵌套传递的结构体载荷（CreateApiKeyRequest），
 * 其内部字段由 serde 按 snake_case 反序列化。
 */
const mocks = vi.hoisted(() => ({ invoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({ invoke: mocks.invoke }));

import {
  createNewAccountItem,
  createNewApiKeyItem,
  createNewEnvVarItem,
  getItemsByGroup,
  updateExistingApiKeyItem,
} from '@/lib/tauri';

describe('tauri command argument casing', () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
  });

  it('getItemsByGroup passes camelCase groupId', async () => {
    await getItemsByGroup('g1');

    expect(mocks.invoke).toHaveBeenCalledWith('get_items_by_group_cmd', { groupId: 'g1' });
  });

  it('createNewAccountItem passes camelCase groupId', async () => {
    await createNewAccountItem('g1', 'T', 'u', 'p');

    expect(mocks.invoke).toHaveBeenCalledWith('create_new_account_item', {
      groupId: 'g1',
      title: 'T',
      username: 'u',
      password: 'p',
      website: undefined,
      notes: undefined,
    });
  });

  it('createNewEnvVarItem passes camelCase groupId', async () => {
    await createNewEnvVarItem('g1', 'T', []);

    expect(mocks.invoke).toHaveBeenCalledWith('create_new_env_var_item', {
      groupId: 'g1',
      title: 'T',
      variables: [],
      notes: undefined,
    });
  });

  it('createNewApiKeyItem nests a snake_case request struct', async () => {
    await createNewApiKeyItem('g1', 'T', 'K', 'V');

    expect(mocks.invoke).toHaveBeenCalledWith('create_new_api_key_item', {
      request: {
        group_id: 'g1',
        title: 'T',
        key_name: 'K',
        key_value: 'V',
        endpoint: undefined,
        auth_method: undefined,
        notes: undefined,
      },
    });
  });

  it('updateExistingApiKeyItem passes camelCase flat args', async () => {
    await updateExistingApiKeyItem('id', 'T', 'K');

    expect(mocks.invoke).toHaveBeenCalledWith('update_existing_api_key_item', {
      id: 'id',
      title: 'T',
      keyName: 'K',
      keyValue: undefined,
      endpoint: undefined,
      authMethod: undefined,
      notes: undefined,
    });
  });
});
