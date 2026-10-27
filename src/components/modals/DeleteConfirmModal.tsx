import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { useI18n } from '@/i18n';

export function DeleteConfirmModal() {
  const { t } = useI18n();
  const target = useUIStore((s) => s.deleteConfirmTarget);
  const close = useUIStore((s) => s.closeDeleteConfirm);
  const deleteGroup = useVaultStore((s) => s.deleteGroup);
  const deleteItem = useVaultStore((s) => s.deleteItem);
  const items = useVaultStore((s) => s.items);

  if (!target) return null;

  const itemCount = target.type === 'group' 
    ? items.filter((i) => i.group_id === target.id).length 
    : 0;

  const typeLabel = target.type === 'group' ? t('group.typeGroup') : t('group.typeItem');

  const handleDelete = async () => {
    if (target.type === 'group') {
      await deleteGroup(target.id);
    } else {
      await deleteItem(target.id);
    }
    close();
  };

  return (
    <Dialog open={!!target} onClose={close} title={t('group.deleteTitle', { type: typeLabel })}>
      <p className="mb-4 text-gray-600 dark:text-gray-300">
        {t('group.deleteBody', { name: target.name })}
        {itemCount > 0 && (
          <span className="block mt-2 font-medium text-red-600 dark:text-red-400">
            {t('group.deleteItemsWarning', { count: itemCount })}
          </span>
        )}
      </p>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={close}>
          {t('common.cancel')}
        </Button>
        <Button variant="destructive" onClick={handleDelete}>
          {t('common.delete')}
        </Button>
      </div>
    </Dialog>
  );
}
