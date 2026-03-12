import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function EditGroupModal() {
  const editingGroupId = useUIStore((s) => s.editingGroupId);
  const close = useUIStore((s) => s.closeEditGroupModal);
  const groups = useVaultStore((s) => s.groups);
  const updateGroup = useVaultStore((s) => s.updateGroup);
  const [name, setName] = useState('');
  const [icon, setIcon] = useState('📂');

  const editingGroup = groups.find((g) => g.id === editingGroupId);

  useEffect(() => {
    if (editingGroup) {
      setName(editingGroup.name);
      setIcon(editingGroup.icon || '📂');
    }
  }, [editingGroup]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editingGroupId) {
      await updateGroup(editingGroupId, name, icon);
    }
    close();
  };

  const icons = ['📂', '🔒', '💼', '🏠', '💳', '📧', '🎮', '🛒'];

  return (
    <Dialog open={!!editingGroupId} onClose={close} title="Rename Group">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium">Name *</label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            maxLength={50}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Icon</label>
          <div className="flex gap-2">
            {icons.map((i) => (
              <button
                key={i}
                type="button"
                onClick={() => setIcon(i)}
                className={`rounded p-2 text-xl ${icon === i ? 'bg-primary-100' : 'hover:bg-gray-100'}`}
              >
                {i}
              </button>
            ))}
          </div>
        </div>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={close}>
            Cancel
          </Button>
          <Button type="submit">Save</Button>
        </div>
      </form>
    </Dialog>
  );
}