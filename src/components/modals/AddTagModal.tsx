import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { useI18n } from '@/i18n';

export function AddTagModal() {
  const { t } = useI18n();
  const isOpen = useUIStore((s) => s.isAddTagModalOpen);
  const close = useUIStore((s) => s.closeAddTagModal);
  const createTag = useVaultStore((s) => s.createTag);
  const [name, setName] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (trimmed === '') return;
    await createTag(trimmed);
    setName('');
    close();
  };

  return (
    <Dialog open={isOpen} onClose={close} title={t('tags.add')}>
      <form onSubmit={handleSubmit} className="space-y-4">
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t('tags.namePlaceholder')}
          autoFocus
          maxLength={30}
        />
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={close}>
            {t('common.cancel')}
          </Button>
          <Button type="submit">{t('tags.create')}</Button>
        </div>
      </form>
    </Dialog>
  );
}
