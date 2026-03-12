import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function AddEditItemModal() {
  const isOpen = useUIStore((s) => s.isAddItemModalOpen || s.editingItemId !== null);
  const editingItemId = useUIStore((s) => s.editingItemId);
  const closeAddItemModal = useUIStore((s) => s.closeAddItemModal);
  const closeEditItemModal = useUIStore((s) => s.closeEditItemModal);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const groups = useVaultStore((s) => s.groups);
  const createItem = useVaultStore((s) => s.createItem);
  const updateItem = useVaultStore((s) => s.updateItem);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);

  const [form, setForm] = useState({
    groupId: '',
    title: '',
    username: '',
    password: '',
    website: '',
    notes: '',
  });

  useEffect(() => {
    if (editingItemId && selectedItem) {
      setForm({
        groupId: selectedItem.group_id,
        title: selectedItem.title,
        username: selectedItem.username,
        password: '',
        website: selectedItem.website || '',
        notes: selectedItem.notes || '',
      });
    } else {
      setForm({
        groupId: selectedGroupId || groups[0]?.id || '',
        title: '',
        username: '',
        password: '',
        website: '',
        notes: '',
      });
    }
  }, [editingItemId, selectedItem, selectedGroupId, groups]);

  const handleClose = () => {
    closeAddItemModal();
    closeEditItemModal();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editingItemId) {
      await updateItem(editingItemId, {
        title: form.title,
        username: form.username,
        password: form.password || undefined,
        website: form.website || undefined,
        notes: form.notes || undefined,
      });
    } else {
      await createItem({
        groupId: form.groupId,
        title: form.title,
        username: form.username,
        password: form.password,
        website: form.website || undefined,
        notes: form.notes || undefined,
      });
    }
    handleClose();
  };

  return (
    <Dialog open={isOpen} onClose={handleClose} title={editingItemId ? 'Edit Item' : 'Add Item'}>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium">Group</label>
          <select
            value={form.groupId}
            onChange={(e) => setForm({ ...form, groupId: e.target.value })}
            className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
            disabled={!!editingItemId}
          >
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.icon} {g.name}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Title *</label>
          <Input
            value={form.title}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
            required
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Username *</label>
          <Input
            value={form.username}
            onChange={(e) => setForm({ ...form, username: e.target.value })}
            required
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">
            Password {editingItemId ? '(leave blank to keep)' : '*'}
          </label>
          <Input
            type="password"
            value={form.password}
            onChange={(e) => setForm({ ...form, password: e.target.value })}
            required={!editingItemId}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Website</label>
          <Input
            value={form.website}
            onChange={(e) => setForm({ ...form, website: e.target.value })}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Notes</label>
          <textarea
            value={form.notes}
            onChange={(e) => setForm({ ...form, notes: e.target.value })}
            className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
            rows={3}
          />
        </div>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit">{editingItemId ? 'Save' : 'Add'}</Button>
        </div>
      </form>
    </Dialog>
  );
}