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

  const isGroup = target.type === 'group';

  const itemCount = isGroup ? items.filter((i) => i.group_id === target.id).length : 0;

  const handleDelete = async () => {
    if (target.type === 'group') {
      await deleteGroup(target.id);
    } else {
      await deleteItem(target.id);
    }
    close();
  };

  return (
    <Dialog
      open={!!target}
      onClose={close}
      title={isGroup ? t('trash.deleteGroupTitle') : t('trash.deleteItemTitle')}
    >
      <p className="mb-4 text-gray-600 dark:text-gray-300">
        {isGroup
          ? t('trash.deleteGroupBody', { name: target.name })
          : t('trash.deleteItemBody', { name: target.name })}
        {isGroup && itemCount > 0 && (
          <span className="block mt-2 font-medium text-red-600 dark:text-red-400">
            {t('trash.deleteGroupItemsWarning', { count: itemCount })}
          </span>
        )}
      </p>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={close}>
          {t('common.cancel')}
        </Button>
        <Button variant="destructive" onClick={handleDelete}>
          {isGroup ? t('common.delete') : t('trash.moveToTrash')}
        </Button>
      </div>
    </Dialog>
  );
}
