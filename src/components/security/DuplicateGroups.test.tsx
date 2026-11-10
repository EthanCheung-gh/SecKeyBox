import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { DuplicateGroups } from '@/components/security/DuplicateGroups';
import { useVaultStore } from '@/stores/vault';
import { useSettingsStore } from '@/stores/settings';
import { archiveDuplicates, findDuplicates, getAllItems, getTrashCount } from '@/lib/tauri';
import type { DuplicateGroup, MergeReport } from '@/types';

vi.mock('@/lib/tauri', () => ({
  findDuplicates: vi.fn(),
  archiveDuplicates: vi.fn(),
  getAllItems: vi.fn(),
  getTrashCount: vi.fn(),
}));

const group: DuplicateGroup = {
  key: 'octocat|github.com',
  type: 'account',
  items: [
    {
      id: 'item-old',
      title: 'GitHub',
      group_id: 'group-work',
      created_at: 1_600_000_000,
      updated_at: 1_600_000_000,
    },
    {
      id: 'item-new',
      title: 'GitHub copy',
      group_id: 'group-work',
      created_at: 1_700_000_000,
      updated_at: 1_700_000_000,
    },
  ],
};

const report: MergeReport = {
  kept_id: 'item-old',
  moved_tags: 1,
  moved_attachments: 0,
  moved_custom_fields: 2,
  moved_history: 3,
  trashed: 1,
};

describe('DuplicateGroups', () => {
  beforeEach(() => {
    useSettingsStore.setState({ language: 'en' });
    vi.mocked(findDuplicates).mockResolvedValue([group]);
    vi.mocked(archiveDuplicates).mockResolvedValue(report);
    vi.mocked(getAllItems).mockResolvedValue([]);
    vi.mocked(getTrashCount).mockResolvedValue(1);

    useVaultStore.setState({
      groups: [
        {
          id: 'group-work',
          name: 'Work',
          sort_order: 0,
          created_at: 1_600_000_000,
          updated_at: 1_600_000_000,
        },
      ],
      error: null,
    });
  });

  it('lists the group, keeps the oldest member by default and archives the rest', async () => {
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);

    render(<DuplicateGroups />);

    expect(await screen.findByText('GitHub copy')).toBeInTheDocument();
    // Group header: localized type + member count, group name resolved from the store.
    expect(screen.getByText('Account')).toBeInTheDocument();
    expect(screen.getByText('2 items')).toBeInTheDocument();
    expect(screen.getAllByText('Work')).toHaveLength(2);

    const radios = screen.getAllByRole('radio') as HTMLInputElement[];
    expect(radios).toHaveLength(2);
    // Backend orders members oldest first, so the first radio is pre-selected.
    expect(radios[0]).toBeChecked();
    expect(radios[1]).not.toBeChecked();

    fireEvent.click(
      screen.getByRole('button', { name: 'Keep selected, move rest to trash' })
    );

    expect(confirm).toHaveBeenCalledWith(
      'Keep "GitHub" and move the other 1 item(s) to the trash? Metadata will be copied to the kept item.'
    );
    await waitFor(() =>
      expect(archiveDuplicates).toHaveBeenCalledWith('item-old', ['item-new'])
    );
    expect(await screen.findByText(/^Merged: 1 to trash/)).toBeInTheDocument();
    expect(screen.getByText(/1 tags · 0 attachments · 2 custom fields · 3 history entries/)).toBeInTheDocument();
  });

  it('does not archive when the confirmation is declined', async () => {
    vi.spyOn(window, 'confirm').mockReturnValue(false);

    render(<DuplicateGroups />);

    fireEvent.click(
      await screen.findByRole('button', { name: 'Keep selected, move rest to trash' })
    );

    expect(archiveDuplicates).not.toHaveBeenCalled();
  });

  it('shows the empty state when nothing is duplicated', async () => {
    vi.mocked(findDuplicates).mockResolvedValue([]);

    render(<DuplicateGroups />);

    expect(await screen.findByText('No duplicate items found.')).toBeInTheDocument();
  });
});
