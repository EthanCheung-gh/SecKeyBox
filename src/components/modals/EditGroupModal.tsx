import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { useI18n } from '@/i18n';

export function EditGroupModal() {
  const { t } = useI18n();
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
    <Dialog open={!!editingGroupId} onClose={close} title={t('group.renameTitle')}>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium">{t('group.nameLabel')}</label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            maxLength={50}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">{t('common.icon')}</label>
          <div className="flex gap-2">
            {icons.map((i) => (
              <button
                key={i}
                type="button"
                onClick={() => setIcon(i)}
                className={`rounded p-2 text-xl ${icon === i ? 'bg-primary-100 dark:bg-primary-900' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}`}
              >
                {i}
              </button>
            ))}
          </div>
        </div>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={close}>
            {t('common.cancel')}
          </Button>
          <Button type="submit">{t('common.save')}</Button>
        </div>
      </form>
    </Dialog>
  );
}
