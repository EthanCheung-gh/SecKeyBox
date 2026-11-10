import { beforeEach, describe, expect, it, vi } from 'vitest';

/**
 * 回归测试：更新后的重启走 Rust 命令。
 *
 * Linux AppImage 上旧的挂载点的 LD_LIBRARY_PATH/APPDIR 会泄漏到重新启动的
 * 进程，导致 /usr/bin/env 加载到不兼容的 libsystemd。因此重启不再调用
 * plugin-process 的 relaunch，而是调用 `restart_app`：由 Rust 侧先清理
 * AppImage 环境变量再重启。
 */
const mocks = vi.hoisted(() => ({ invoke: vi.fn(), check: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({ invoke: mocks.invoke }));
vi.mock('@tauri-apps/plugin-updater', () => ({ check: mocks.check }));

import { restartApp } from '@/lib/updater';

describe('updater restart', () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
  });

  it('restartApp invokes the restart_app command', async () => {
    mocks.invoke.mockResolvedValue(undefined);

    await restartApp();

    expect(mocks.invoke).toHaveBeenCalledWith('restart_app');
  });
});
