import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { useI18n } from '@/i18n';

export function AddGroupModal() {
  const { t } = useI18n();
  const isOpen = useUIStore((s) => s.isAddGroupModalOpen);
  const close = useUIStore((s) => s.closeAddGroupModal);
  const createGroup = useVaultStore((s) => s.createGroup);
  const [name, setName] = useState('');
  const [icon, setIcon] = useState('📂');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await createGroup(name, icon);
    setName('');
    setIcon('📂');
    close();
  };

  const icons = ['📂', '🔒', '💼', '🏠', '💳', '📧', '🎮', '🛒'];

  return (
    <Dialog open={isOpen} onClose={close} title={t('group.addTitle')}>
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
                className={`rounded p-2 text-xl ${icon === i ? 'bg-primary-100' : 'hover:bg-gray-100'}`}
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
          <Button type="submit">{t('common.add')}</Button>
        </div>
      </form>
    </Dialog>
  );
}
