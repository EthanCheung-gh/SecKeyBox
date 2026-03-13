import { Copy, Eye, EyeOff, Edit, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { copyToClipboard } from '@/lib/tauri';
import { maskKeyValue, formatRotationDate } from '@/lib/utils';
import type { AccountItemDetail, ApiKeyItemDetail, EnvVarItemDetail } from '@/types';

export function DetailPanel() {
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const openEditItemModal = useUIStore((s) => s.openEditItemModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const [showPassword, setShowPassword] = useState(false);
  const [showKeyValue, setShowKeyValue] = useState(false);
  const [showEnvVarValues, setShowEnvVarValues] = useState<Record<number, boolean>>({});

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

  const renderAccountDetail = () => {
    const item = selectedItem as AccountItemDetail;
    return (
      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Username</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">{item.username}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.username)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Password</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">
              {showPassword ? item.password : '••••••••'}
            </span>
            <Button variant="ghost" size="sm" onClick={() => setShowPassword(!showPassword)}>
              {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.password)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {item.website && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Website</label>
            <a
              href={item.website}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary-600 hover:underline"
            >
              {item.website}
            </a>
          </div>
        )}

        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderApiKeyDetail = () => {
    const item = selectedItem as ApiKeyItemDetail;
    return (
      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Key Name</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">{item.key_name}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.key_name)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Key Value</label>
          <div className="flex items-center gap-2">
            <span className="font-medium font-mono text-sm">
              {showKeyValue ? item.key_value : maskKeyValue(item.key_value)}
            </span>
            <Button variant="ghost" size="sm" onClick={() => setShowKeyValue(!showKeyValue)}>
              {showKeyValue ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.key_value)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {item.endpoint && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Endpoint</label>
            <div className="flex items-center gap-2">
              <a
                href={item.endpoint}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary-600 hover:underline"
              >
                {item.endpoint}
              </a>
              <Button variant="ghost" size="sm" onClick={() => handleCopy(item.endpoint!)}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </div>
        )}

        {item.auth_method && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Auth Method</label>
            <div className="flex items-center gap-2">
              <span className="font-medium capitalize">{item.auth_method.replace('_', ' ')}</span>
              <Button variant="ghost" size="sm" onClick={() => handleCopy(item.auth_method!)}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </div>
        )}

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Last Rotated</label>
          <span className="text-gray-700">{formatRotationDate(item.rotation_date)}</span>
        </div>

        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderEnvVarDetail = () => {
    const item = selectedItem as EnvVarItemDetail;
    return (
      <div className="space-y-4">
        <div>
          <label className="mb-2 block text-sm font-medium text-gray-500">
            Variables ({item.variables.length})
          </label>
          <div className="space-y-2">
            {item.variables.map((v, i) => (
              <div key={i} className="rounded-md border border-gray-200 p-3">
                <div className="mb-1 flex items-center justify-between">
                  <span className="font-mono text-sm font-medium text-gray-700">{v.key}</span>
                  <Button variant="ghost" size="sm" onClick={() => handleCopy(v.key)}>
                    <Copy className="h-3 w-3" />
                  </Button>
                </div>
                <div className="flex items-center gap-2">
                  <span className="font-mono text-sm">
                    {showEnvVarValues[i] ? v.value : '••••••••'}
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setShowEnvVarValues({ ...showEnvVarValues, [i]: !showEnvVarValues[i] })}
                  >
                    {showEnvVarValues[i] ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
                  </Button>
                  <Button variant="ghost" size="sm" onClick={() => handleCopy(v.value)}>
                    <Copy className="h-3 w-3" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </div>

        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderDetail = () => {
    switch (selectedItem.type) {
      case 'account':
        return renderAccountDetail();
      case 'api_key':
        return renderApiKeyDetail();
      case 'env_var':
        return renderEnvVarDetail();
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-1 flex-col bg-white p-6">
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-xl font-semibold">{selectedItem.title}</h1>
          <span className="text-sm text-gray-500 capitalize">{selectedItem.type.replace('_', ' ')}</span>
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

      {renderDetail()}
    </div>
  );
}