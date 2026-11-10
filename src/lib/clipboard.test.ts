import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { copySecret } from '@/lib/clipboard';
import { clearClipboard, copyToClipboard } from '@/lib/tauri';
import { useSettingsStore } from '@/stores/settings';

vi.mock('@/lib/tauri', () => ({
  copyToClipboard: vi.fn(() => Promise.resolve()),
  clearClipboard: vi.fn(() => Promise.resolve()),
}));

describe('copySecret', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    useSettingsStore.setState({ clipboardClearSeconds: 30 });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('clears the clipboard after the configured delay', async () => {
    await copySecret('s3cret');

    expect(copyToClipboard).toHaveBeenCalledWith('s3cret', undefined);
    expect(clearClipboard).not.toHaveBeenCalled();

    vi.advanceTimersByTime(29_999);
    expect(clearClipboard).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(clearClipboard).toHaveBeenCalledTimes(1);
  });

  it('never clears the clipboard when the setting is 0', async () => {
    useSettingsStore.setState({ clipboardClearSeconds: 0 });

    await copySecret('s3cret');
    vi.advanceTimersByTime(10 * 60 * 1000);

    expect(copyToClipboard).toHaveBeenCalledTimes(1);
    expect(clearClipboard).not.toHaveBeenCalled();
  });

  it('passes the copy context through to the backend', async () => {
    const context = { itemId: 'item-1', itemTitle: 'GitHub', field: 'password' };

    await copySecret('s3cret', context);

    expect(copyToClipboard).toHaveBeenCalledWith('s3cret', context);
  });
});
