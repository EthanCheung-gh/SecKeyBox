import { Copy, Eye, EyeOff, Edit, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { copyToClipboard } from '@/lib/tauri';

export function DetailPanel() {
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const openEditItemModal = useUIStore((s) => s.openEditItemModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const [showPassword, setShowPassword] = useState(false);

  if (!selectedItem) {
    return (
      <div className="flex h-full flex-1 items-center justify-center bg-gray-50">
        <p className="text-gray-400">Select an item to view details</p>
      </div>
    );
  }

  const handleCopy = async (text: string) => {
    await copyToClipboard(text);
    setTimeout(async () => {
      await import('@/lib/tauri').then(api => api.clearClipboard());
    }, 30000);
  };

  return (
    <div className="flex h-full flex-1 flex-col bg-white p-6">
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-xl font-semibold">{selectedItem.title}</h1>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => openEditItemModal(selectedItem.id)}>
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => openDeleteConfirm('item', selectedItem.id, selectedItem.title)}
          >
            <Trash2 className="h-4 w-4 text-red-500" />
          </Button>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Username</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">{selectedItem.username}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(selectedItem.username)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Password</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">
              {showPassword ? selectedItem.password : '••••••••'}
            </span>
            <Button variant="ghost" size="sm" onClick={() => setShowPassword(!showPassword)}>
              {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(selectedItem.password)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {selectedItem.website && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Website</label>
            <a
              href={selectedItem.website}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary-600 hover:underline"
            >
              {selectedItem.website}
            </a>
          </div>
        )}

        {selectedItem.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700">{selectedItem.notes}</p>
          </div>
        )}
      </div>
    </div>
  );
}