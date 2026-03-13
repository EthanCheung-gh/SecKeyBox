import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function DeleteConfirmModal() {
  const target = useUIStore((s) => s.deleteConfirmTarget);
  const close = useUIStore((s) => s.closeDeleteConfirm);
  const deleteGroup = useVaultStore((s) => s.deleteGroup);
  const deleteItem = useVaultStore((s) => s.deleteItem);
  const items = useVaultStore((s) => s.items);

  if (!target) return null;

  const itemCount = target.type === 'group' 
    ? items.filter((i) => i.group_id === target.id).length 
    : 0;

  const handleDelete = async () => {
    if (target.type === 'group') {
      await deleteGroup(target.id);
    } else {
      await deleteItem(target.id);
    }
    close();
  };

  return (
    <Dialog open={!!target} onClose={close} title={`Delete ${target.type}?`}>
      <p className="mb-4 text-gray-600 dark:text-gray-300">
        Are you sure you want to delete "{target.name}"?
        {itemCount > 0 && (
          <span className="block mt-2 font-medium text-red-600 dark:text-red-400">
            This will permanently delete {itemCount} item{itemCount !== 1 ? 's' : ''} in this group.
          </span>
        )}
      </p>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={close}>
          Cancel
        </Button>
        <Button variant="destructive" onClick={handleDelete}>
          Delete
        </Button>
      </div>
    </Dialog>
  );
}