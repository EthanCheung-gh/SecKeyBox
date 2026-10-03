import { Copy, Eye, EyeOff, Edit, Trash2, Star } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { copyToClipboard } from '@/lib/tauri';
import { maskKeyValue, formatRotationDate, formatExpiryDate } from '@/lib/utils';
import type { AccountItemDetail, ApiKeyItemDetail, EnvVarItemDetail, DatabaseItemDetail, SshItemDetail, CloudItemDetail, LicenseItemDetail, SmtpItemDetail } from '@/types';

export function DetailPanel() {
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const toggleItemFavorite = useVaultStore((s) => s.toggleItemFavorite);
  const openEditItemModal = useUIStore((s) => s.openEditItemModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const [showPassword, setShowPassword] = useState(false);
  const [showKeyValue, setShowKeyValue] = useState(false);
  const [showEnvVarValues, setShowEnvVarValues] = useState<Record<number, boolean>>({});

  if (!selectedItem) {
    return (
      <div className="flex h-full flex-1 items-center justify-center bg-gray-50 dark:bg-gray-900">
        <p className="text-gray-400 dark:text-gray-500">Select an item to view details</p>
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
          <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Username</label>
          <div className="flex items-center gap-2">
            <span className="font-medium dark:text-gray-100">{item.username}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.username)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Password</label>
          <div className="flex items-center gap-2">
            <span className="font-medium dark:text-gray-100">
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
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Website</label>
            <a
              href={item.website}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary-600 hover:underline dark:text-primary-400"
            >
              {item.website}
            </a>
          </div>
        )}

        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
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
          <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Key Name</label>
          <div className="flex items-center gap-2">
            <span className="font-medium dark:text-gray-100">{item.key_name}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.key_name)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Key Value</label>
          <div className="flex items-center gap-2">
            <span className="font-medium font-mono text-sm dark:text-gray-100">
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
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Endpoint</label>
            <div className="flex items-center gap-2">
              <a
                href={item.endpoint}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary-600 hover:underline dark:text-primary-400"
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
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Auth Method</label>
            <div className="flex items-center gap-2">
              <span className="font-medium capitalize dark:text-gray-100">{item.auth_method.replace('_', ' ')}</span>
              <Button variant="ghost" size="sm" onClick={() => handleCopy(item.auth_method!)}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </div>
        )}

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Last Rotated</label>
          <span className="text-gray-700 dark:text-gray-300">{formatRotationDate(item.rotation_date)}</span>
        </div>

        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
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
          <label className="mb-2 block text-sm font-medium text-gray-500 dark:text-gray-400">
            Variables ({item.variables.length})
          </label>
          <div className="space-y-2">
            {item.variables.map((v, i) => (
              <div key={i} className="rounded-md border border-gray-200 p-3 dark:border-gray-600">
                <div className="mb-1 flex items-center justify-between">
                  <span className="font-mono text-sm font-medium text-gray-700 dark:text-gray-300">{v.key}</span>
                  <Button variant="ghost" size="sm" onClick={() => handleCopy(v.key)}>
                    <Copy className="h-3 w-3" />
                  </Button>
                </div>
                <div className="flex items-center gap-2">
                  <span className="font-mono text-sm dark:text-gray-100">
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
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderField = (label: string, value: string, mono = false, secret = false) => (
    <div>
      <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">{label}</label>
      <div className="flex items-center gap-2">
        <span className={`font-medium dark:text-gray-100 ${mono ? 'font-mono text-sm' : ''}`}>
          {value === '' ? '—' : secret ? '••••••••' : value}
        </span>
      </div>
    </div>
  );

  const renderSecretField = (label: string, value: string | undefined, shown: boolean, onToggle: () => void, suffix?: string) => (
    <div>
      <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">{label}</label>
      <div className="flex items-center gap-2">
        <span className="font-mono text-sm font-medium dark:text-gray-100">
          {value === undefined || value === '' ? '—' : shown ? value : `••••••••${suffix ?? ''}`}
        </span>
        {value !== undefined && value !== '' && (
          <>
            <Button variant="ghost" size="sm" onClick={onToggle}>
              {shown ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(value)}>
              <Copy className="h-4 w-4" />
            </Button>
          </>
        )}
      </div>
    </div>
  );

  const renderDatabaseDetail = () => {
    const item = selectedItem as DatabaseItemDetail;
    return (
      <div className="space-y-4">
        {renderField('Database Type', item.db_type)}
        {renderField('Host', item.port ? `${item.host}:${item.port}` : item.host, true)}
        {renderField('Database', item.database_name ?? '', true)}
        {renderField('Username', item.username ?? '')}
        {renderSecretField('Password', item.password, showPassword, () => setShowPassword(!showPassword))}
        {item.connection_url && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Connection URL</label>
            <div className="flex items-center gap-2">
              <span className="break-all font-mono text-sm dark:text-gray-100">{item.connection_url}</span>
              <Button variant="ghost" size="sm" onClick={() => handleCopy(item.connection_url!)}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </div>
        )}
        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderSshDetail = () => {
    const item = selectedItem as SshItemDetail;
    return (
      <div className="space-y-4">
        {renderField('Host', item.port ? `${item.host}:${item.port}` : item.host, true)}
        {renderField('Username', item.username)}
        {renderSecretField('Password', item.password, showPassword, () => setShowPassword(!showPassword))}
        {item.key_path && renderField('Private Key Path', item.key_path, true)}
        {renderSecretField('Key Passphrase', item.passphrase, showKeyValue, () => setShowKeyValue(!showKeyValue))}
        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderCloudDetail = () => {
    const item = selectedItem as CloudItemDetail;
    return (
      <div className="space-y-4">
        {renderField('Provider', item.provider)}
        {renderField('Access Key ID', item.access_key_id, true)}
        {renderSecretField('Secret', item.secret, showKeyValue, () => setShowKeyValue(!showKeyValue), item.secret.slice(-4))}
        {item.region && renderField('Region', item.region)}
        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderLicenseDetail = () => {
    const item = selectedItem as LicenseItemDetail;
    return (
      <div className="space-y-4">
        {renderField('Software', item.software_name)}
        {renderSecretField('License Key', item.license_key, showKeyValue, () => setShowKeyValue(!showKeyValue))}
        {item.bound_email && renderField('Bound Email', item.bound_email)}
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Expiry Date</label>
          <span className="text-gray-700 dark:text-gray-300">{formatExpiryDate(item.expiry_date)}</span>
        </div>
        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderSmtpDetail = () => {
    const item = selectedItem as SmtpItemDetail;
    return (
      <div className="space-y-4">
        {renderField('Host', item.port ? `${item.host}:${item.port}` : item.host, true)}
        {item.encryption && renderField('Encryption', item.encryption.toUpperCase())}
        {renderField('Username', item.username ?? '')}
        {renderSecretField('Password', item.password, showPassword, () => setShowPassword(!showPassword))}
        {item.from_address && renderField('From Address', item.from_address)}
        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-400">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700 dark:text-gray-300">{item.notes}</p>
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
      case 'database':
        return renderDatabaseDetail();
      case 'ssh':
        return renderSshDetail();
      case 'cloud':
        return renderCloudDetail();
      case 'license':
        return renderLicenseDetail();
      case 'smtp':
        return renderSmtpDetail();
      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-1 flex-col bg-white p-6 dark:bg-gray-900">
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-xl font-semibold dark:text-gray-100">{selectedItem.title}</h1>
          <span className="text-sm text-gray-500 capitalize dark:text-gray-400">{selectedItem.type.replace('_', ' ')}</span>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => toggleItemFavorite(selectedItem.id)}
          >
            <Star className={`h-4 w-4 ${selectedItem.is_favorite ? 'fill-yellow-400 text-yellow-400' : ''}`} />
          </Button>
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