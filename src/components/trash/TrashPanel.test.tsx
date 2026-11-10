import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import { TrashPanel } from '@/components/trash/TrashPanel';
import { useVaultStore } from '@/stores/vault';
import {
  emptyTrash,
  getGroups,
  getSecuritySettings,
  getAllItems,
  getTrashCount,
  listTrash,
  purgeTrashItem,
  restoreTrashItem,
} from '@/lib/tauri';
import type { TrashItemSummary } from '@/types';

vi.mock('@/lib/tauri', () => ({
  getSecuritySettings: vi.fn(),
  listTrash: vi.fn(),
  getTrashCount: vi.fn(),
  restoreTrashItem: vi.fn(),
  purgeTrashItem: vi.fn(),
  emptyTrash: vi.fn(),
  getAllItems: vi.fn(),
  getGroups: vi.fn(),
}));

const trashedItem: TrashItemSummary = {
  id: 'item-1',
  title: 'GitHub token',
  type: 'account',
  icon: '🐙',
  deleted_at: 1_700_000_000,
  original_group_id: 'built-in-accounts',
  original_group_name: 'Accounts',
};

describe('TrashPanel', () => {
  beforeEach(() => {
    vi.mocked(getSecuritySettings).mockResolvedValue({
      auto_lock_minutes: 5,
      trash_retention_days: 30,
    });
    vi.mocked(listTrash).mockResolvedValue([trashedItem]);
    vi.mocked(getTrashCount).mockResolvedValue(1);
    vi.mocked(restoreTrashItem).mockResolvedValue(undefined);
    vi.mocked(purgeTrashItem).mockResolvedValue(undefined);
    vi.mocked(emptyTrash).mockResolvedValue(1);
    vi.mocked(getAllItems).mockResolvedValue([]);
    vi.mocked(getGroups).mockResolvedValue([]);

    useVaultStore.setState({
      trashItems: [trashedItem],
      trashCount: 1,
      error: null,
    });
  });

  it('lists trashed items with the retention hint and restores one', async () => {
    render(<TrashPanel />);

    expect(await screen.findByText(/GitHub token/)).toBeInTheDocument();
    expect(
      await screen.findByText('Items are kept for 30 days, then removed automatically.')
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Restore' }));

    expect(restoreTrashItem).toHaveBeenCalledWith('item-1');
  });

  it('asks for confirmation before purging an item', async () => {
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false);

    render(<TrashPanel />);

    fireEvent.click(await screen.findByRole('button', { name: 'Delete forever' }));

    expect(confirm).toHaveBeenCalled();
    expect(purgeTrashItem).not.toHaveBeenCalled();
  });
});
