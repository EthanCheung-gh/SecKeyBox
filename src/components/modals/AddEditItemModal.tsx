import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { PasswordStrength } from '@/components/ui/password-strength';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import type { EnvVarPair } from '@/types';

// 统一表单模型：所有类型的字段放在一个 state 里，未用到的字段保持空串。
// 密文类字段（password/keyValue/secret/licenseKey/passphrase）在编辑模式下
// 留空表示“保持原值”（提交时转 undefined），与后端 None = keep 语义一致。
const emptyForm = {
  groupId: '',
  title: '',
  // account
  username: '',
  password: '',
  website: '',
  // api_key
  keyName: '',
  keyValue: '',
  endpoint: '',
  authMethod: '',
  // env_var
  variables: [{ key: '', value: '' }] as EnvVarPair[],
  // database
  dbType: '',
  host: '',
  port: '',
  databaseName: '',
  connectionUrl: '',
  // ssh
  keyPath: '',
  passphrase: '',
  // cloud
  provider: '',
  accessKeyId: '',
  secret: '',
  region: '',
  // license
  softwareName: '',
  licenseKey: '',
  boundEmail: '',
  expiryDate: '',
  // smtp
  encryption: '',
  fromAddress: '',
  notes: '',
};

type ItemForm = typeof emptyForm;

const ItemTypeSelect = ({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: 'account' | 'api_key' | 'env_var' | 'database' | 'ssh' | 'cloud' | 'license' | 'smtp') => void;
}) => (
  <select
    value={value}
    onChange={(e) => onChange(e.target.value as Parameters<typeof onChange>[0])}
    className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
  >
    <option value="account">Account</option>
    <option value="api_key">API Key</option>
    <option value="env_var">Environment Variables</option>
    <option value="database">Database</option>
    <option value="ssh">SSH Server</option>
    <option value="cloud">Cloud Credential</option>
    <option value="license">License</option>
    <option value="smtp">Email (SMTP)</option>
  </select>
);

const Field = ({ label, children }: { label: string; children: React.ReactNode }) => (
  <div>
    <label className="mb-1 block text-sm font-medium">{label}</label>
    {children}
  </div>
);

const textAreaClass =
  'w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100';
const selectClass = textAreaClass;

const SECRET_HINT_KEEP = '(leave blank to keep)';

export function AddEditItemModal() {
  const isOpen = useUIStore((s) => s.isAddItemModalOpen || s.editingItemId !== null);
  const editingItemId = useUIStore((s) => s.editingItemId);
  const closeAddItemModal = useUIStore((s) => s.closeAddItemModal);
  const closeEditItemModal = useUIStore((s) => s.closeEditItemModal);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const groups = useVaultStore((s) => s.groups);
  const vault = useVaultStore();
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);
  const newItemType = useUIStore((s) => s.newItemType);
  const setNewItemType = useUIStore((s) => s.setNewItemType);

  const [form, setForm] = useState<ItemForm>(emptyForm);
  const set = (patch: Partial<ItemForm>) => setForm({ ...form, ...patch });

  const isEditing = editingItemId !== null;
  const editingType = selectedItem?.type || 'account';
  const currentType = isEditing ? editingType : newItemType;

  useEffect(() => {
    if (isEditing && selectedItem) {
      const base = {
        groupId: selectedItem.group_id,
        title: selectedItem.title,
        notes: selectedItem.notes || '',
        variables: [{ key: '', value: '' }] as EnvVarPair[],
        username: '',
        password: '',
        website: '',
        keyName: '',
        keyValue: '',
        endpoint: '',
        authMethod: '',
        dbType: '',
        host: '',
        port: '',
        databaseName: '',
        connectionUrl: '',
        keyPath: '',
        passphrase: '',
        provider: '',
        accessKeyId: '',
        secret: '',
        region: '',
        softwareName: '',
        licenseKey: '',
        boundEmail: '',
        expiryDate: '',
        encryption: '',
        fromAddress: '',
      };
      switch (selectedItem.type) {
        case 'account':
          setForm({ ...base, username: selectedItem.username, website: selectedItem.website || '' });
          break;
        case 'api_key':
          setForm({
            ...base,
            keyName: selectedItem.key_name,
            endpoint: selectedItem.endpoint || '',
            authMethod: selectedItem.auth_method || '',
          });
          break;
        case 'env_var':
          setForm((prev) => ({
            ...prev,
            ...base,
            variables:
              selectedItem.variables.length > 0 ? selectedItem.variables : [{ key: '', value: '' }],
          }));
          break;
        case 'database':
          setForm({
            ...base,
            dbType: selectedItem.db_type,
            host: selectedItem.host,
            port: selectedItem.port != null ? String(selectedItem.port) : '',
            databaseName: selectedItem.database_name || '',
            username: selectedItem.username || '',
            connectionUrl: selectedItem.connection_url || '',
          });
          break;
        case 'ssh':
          setForm({
            ...base,
            host: selectedItem.host,
            port: selectedItem.port != null ? String(selectedItem.port) : '',
            username: selectedItem.username,
            keyPath: selectedItem.key_path || '',
          });
          break;
        case 'cloud':
          setForm({
            ...base,
            provider: selectedItem.provider,
            accessKeyId: selectedItem.access_key_id,
            region: selectedItem.region || '',
          });
          break;
        case 'license':
          setForm({
            ...base,
            softwareName: selectedItem.software_name,
            boundEmail: selectedItem.bound_email || '',
            expiryDate:
              selectedItem.expiry_date != null
                ? new Date(selectedItem.expiry_date * 1000).toISOString().slice(0, 10)
                : '',
          });
          break;
        case 'smtp':
          setForm({
            ...base,
            host: selectedItem.host,
            port: selectedItem.port != null ? String(selectedItem.port) : '',
            encryption: selectedItem.encryption || '',
            username: selectedItem.username || '',
            fromAddress: selectedItem.from_address || '',
          });
          break;
      }
    } else {
      setForm({ ...emptyForm, groupId: selectedGroupId || groups[0]?.id || '' });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isEditing, selectedItem, selectedGroupId, groups]);

  const handleClose = () => {
    closeAddItemModal();
    closeEditItemModal();
    setNewItemType('account');
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const id = editingItemId;
    const opt = (v: string) => (v === '' ? undefined : v);
    const secret = (v: string) => (v === '' ? undefined : v); // 空 = 保持原值
    const num = (v: string) => (v === '' ? undefined : Number(v));

    switch (currentType) {
      case 'account':
        if (isEditing) {
          await vault.updateItem(id!, {
            title: form.title,
            username: form.username,
            password: secret(form.password),
            website: opt(form.website),
            notes: opt(form.notes),
          });
        } else {
          await vault.createItem({
            groupId: form.groupId,
            title: form.title,
            username: form.username,
            password: form.password,
            website: opt(form.website),
            notes: opt(form.notes),
          });
        }
        break;
      case 'api_key':
        if (isEditing) {
          await vault.updateApiKeyItem(id!, {
            title: form.title,
            keyName: form.keyName,
            keyValue: secret(form.keyValue),
            endpoint: opt(form.endpoint),
            authMethod: opt(form.authMethod),
            notes: opt(form.notes),
          });
        } else {
          await vault.createApiKeyItem({
            groupId: form.groupId,
            title: form.title,
            keyName: form.keyName,
            keyValue: form.keyValue,
            endpoint: opt(form.endpoint),
            authMethod: opt(form.authMethod),
            notes: opt(form.notes),
          });
        }
        break;
      case 'env_var': {
        const validVars = form.variables.filter((v) => v.key.trim() && v.value.trim());
        if (isEditing) {
          await vault.updateEnvVarItem(id!, {
            title: form.title,
            variables: validVars,
            notes: opt(form.notes),
          });
        } else {
          await vault.createEnvVarItem({
            groupId: form.groupId,
            title: form.title,
            variables: validVars,
            notes: opt(form.notes),
          });
        }
        break;
      }
      case 'database':
        if (isEditing) {
          await vault.updateDatabaseItem(id!, {
            title: form.title,
            dbType: form.dbType,
            host: form.host,
            port: num(form.port),
            databaseName: opt(form.databaseName),
            username: opt(form.username),
            password: secret(form.password),
            connectionUrl: opt(form.connectionUrl),
            notes: opt(form.notes),
          });
        } else {
          await vault.createDatabaseItem({
            groupId: form.groupId,
            title: form.title,
            dbType: form.dbType,
            host: form.host,
            port: num(form.port),
            databaseName: opt(form.databaseName),
            username: opt(form.username),
            password: opt(form.password),
            connectionUrl: opt(form.connectionUrl),
            notes: opt(form.notes),
          });
        }
        break;
      case 'ssh':
        if (isEditing) {
          await vault.updateSshItem(id!, {
            title: form.title,
            host: form.host,
            port: num(form.port),
            username: form.username,
            password: secret(form.password),
            keyPath: opt(form.keyPath),
            passphrase: secret(form.passphrase),
            notes: opt(form.notes),
          });
        } else {
          await vault.createSshItem({
            groupId: form.groupId,
            title: form.title,
            host: form.host,
            port: num(form.port),
            username: form.username,
            password: opt(form.password),
            keyPath: opt(form.keyPath),
            passphrase: opt(form.passphrase),
            notes: opt(form.notes),
          });
        }
        break;
      case 'cloud':
        if (isEditing) {
          await vault.updateCloudItem(id!, {
            title: form.title,
            provider: form.provider,
            accessKeyId: form.accessKeyId,
            secret: secret(form.secret),
            region: opt(form.region),
            notes: opt(form.notes),
          });
        } else {
          await vault.createCloudItem({
            groupId: form.groupId,
            title: form.title,
            provider: form.provider,
            accessKeyId: form.accessKeyId,
            secret: form.secret,
            region: opt(form.region),
            notes: opt(form.notes),
          });
        }
        break;
      case 'license': {
        const expiry = form.expiryDate
          ? Math.floor(new Date(form.expiryDate + 'T00:00:00Z').getTime() / 1000)
          : undefined;
        if (isEditing) {
          await vault.updateLicenseItem(id!, {
            title: form.title,
            softwareName: form.softwareName,
            licenseKey: secret(form.licenseKey),
            boundEmail: opt(form.boundEmail),
            expiryDate: expiry,
            notes: opt(form.notes),
          });
        } else {
          await vault.createLicenseItem({
            groupId: form.groupId,
            title: form.title,
            softwareName: form.softwareName,
            licenseKey: form.licenseKey,
            boundEmail: opt(form.boundEmail),
            expiryDate: expiry,
            notes: opt(form.notes),
          });
        }
        break;
      }
      case 'smtp':
        if (isEditing) {
          await vault.updateSmtpItem(id!, {
            title: form.title,
            host: form.host,
            port: num(form.port),
            encryption: opt(form.encryption),
            username: opt(form.username),
            password: secret(form.password),
            fromAddress: opt(form.fromAddress),
            notes: opt(form.notes),
          });
        } else {
          await vault.createSmtpItem({
            groupId: form.groupId,
            title: form.title,
            host: form.host,
            port: num(form.port),
            encryption: opt(form.encryption),
            username: opt(form.username),
            password: form.password,
            fromAddress: opt(form.fromAddress),
            notes: opt(form.notes),
          });
        }
        break;
    }
    handleClose();
  };

  const addEnvVar = () => set({ variables: [...form.variables, { key: '', value: '' }] });

  const removeEnvVar = (index: number) => {
    if (form.variables.length > 1) {
      set({ variables: form.variables.filter((_, i) => i !== index) });
    }
  };

  const updateEnvVar = (index: number, field: 'key' | 'value', value: string) => {
    const newVars = [...form.variables];
    newVars[index] = { ...newVars[index], [field]: value };
    set({ variables: newVars });
  };

  const renderFields = () => {
    switch (currentType) {
      case 'account':
        return (
          <>
            <Field label="Username *">
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} required />
            </Field>
            <Field label={`Password ${isEditing ? SECRET_HINT_KEEP : '*'}`}>
              <Input
                type="password"
                value={form.password}
                onChange={(e) => set({ password: e.target.value })}
                required={!isEditing}
              />
              <PasswordStrength password={form.password} />
            </Field>
            <Field label="Website">
              <Input value={form.website} onChange={(e) => set({ website: e.target.value })} />
            </Field>
          </>
        );
      case 'api_key':
        return (
          <>
            <Field label="Key Name *">
              <Input
                value={form.keyName}
                onChange={(e) => set({ keyName: e.target.value })}
                required
                placeholder="e.g., OpenAI API Key"
              />
            </Field>
            <Field label={`Key Value ${isEditing ? SECRET_HINT_KEEP : '*'}`}>
              <textarea
                value={form.keyValue}
                onChange={(e) => set({ keyValue: e.target.value })}
                className={textAreaClass + ' font-mono'}
                rows={4}
                required={!isEditing}
                placeholder="Paste your API key here..."
              />
            </Field>
            <Field label="Endpoint URL">
              <Input
                value={form.endpoint}
                onChange={(e) => set({ endpoint: e.target.value })}
                placeholder="https://api.example.com"
              />
            </Field>
            <Field label="Auth Method">
              <select
                value={form.authMethod}
                onChange={(e) => set({ authMethod: e.target.value })}
                className={selectClass}
              >
                <option value="">Select...</option>
                <option value="bearer">Bearer Token</option>
                <option value="api_key">API Key Header</option>
                <option value="basic">Basic Auth</option>
                <option value="custom">Custom</option>
              </select>
            </Field>
          </>
        );
      case 'env_var':
        return (
          <Field label="Variables *">
            <div className="mb-2 flex justify-end">
              <Button type="button" variant="outline" size="sm" onClick={addEnvVar}>
                + Add Variable
              </Button>
            </div>
            <div className="space-y-2">
              {form.variables.map((v, i) => (
                <div key={i} className="flex gap-2">
                  <Input
                    placeholder="KEY"
                    value={v.key}
                    onChange={(e) => updateEnvVar(i, 'key', e.target.value)}
                    className="flex-1 font-mono text-sm"
                  />
                  <Input
                    placeholder="value"
                    value={v.value}
                    onChange={(e) => updateEnvVar(i, 'value', e.target.value)}
                    className="flex-1"
                  />
                  {form.variables.length > 1 && (
                    <Button type="button" variant="ghost" size="sm" onClick={() => removeEnvVar(i)}>
                      ×
                    </Button>
                  )}
                </div>
              ))}
            </div>
          </Field>
        );
      case 'database':
        return (
          <>
            <Field label="Database Type *">
              <select value={form.dbType} onChange={(e) => set({ dbType: e.target.value })} className={selectClass} required>
                <option value="">Select...</option>
                <option value="mysql">MySQL / MariaDB</option>
                <option value="postgresql">PostgreSQL</option>
                <option value="redis">Redis</option>
                <option value="mongodb">MongoDB</option>
                <option value="sqlserver">SQL Server</option>
                <option value="sqlite">SQLite</option>
                <option value="other">Other</option>
              </select>
            </Field>
            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-2">
                <Field label="Host *">
                  <Input value={form.host} onChange={(e) => set({ host: e.target.value })} required placeholder="db.example.com" />
                </Field>
              </div>
              <Field label="Port">
                <Input type="number" value={form.port} onChange={(e) => set({ port: e.target.value })} placeholder="3306" />
              </Field>
            </div>
            <Field label="Database Name">
              <Input value={form.databaseName} onChange={(e) => set({ databaseName: e.target.value })} />
            </Field>
            <Field label="Username">
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} />
            </Field>
            <Field label={`Password ${isEditing ? SECRET_HINT_KEEP : ''}`}>
              <Input
                type="password"
                value={form.password}
                onChange={(e) => set({ password: e.target.value })}
              />
            </Field>
            <Field label="Connection URL">
              <Input
                value={form.connectionUrl}
                onChange={(e) => set({ connectionUrl: e.target.value })}
                placeholder="mysql://user:pass@host:3306/db"
                className="font-mono text-sm"
              />
            </Field>
          </>
        );
      case 'ssh':
        return (
          <>
            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-2">
                <Field label="Host *">
                  <Input value={form.host} onChange={(e) => set({ host: e.target.value })} required placeholder="server.example.com" />
                </Field>
              </div>
              <Field label="Port">
                <Input type="number" value={form.port} onChange={(e) => set({ port: e.target.value })} placeholder="22" />
              </Field>
            </div>
            <Field label="Username *">
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} required placeholder="root" />
            </Field>
            <Field label={`Password ${isEditing ? SECRET_HINT_KEEP : '(leave blank if using key auth)'}`}>
              <Input type="password" value={form.password} onChange={(e) => set({ password: e.target.value })} />
            </Field>
            <Field label="Private Key Path">
              <Input
                value={form.keyPath}
                onChange={(e) => set({ keyPath: e.target.value })}
                placeholder="~/.ssh/id_ed25519"
                className="font-mono text-sm"
              />
            </Field>
            <Field label={`Key Passphrase ${isEditing ? SECRET_HINT_KEEP : ''}`}>
              <Input type="password" value={form.passphrase} onChange={(e) => set({ passphrase: e.target.value })} />
            </Field>
          </>
        );
      case 'cloud':
        return (
          <>
            <Field label="Provider *">
              <select value={form.provider} onChange={(e) => set({ provider: e.target.value })} className={selectClass} required>
                <option value="">Select...</option>
                <option value="aws">AWS</option>
                <option value="gcp">Google Cloud</option>
                <option value="azure">Azure</option>
                <option value="alicloud">Alibaba Cloud</option>
                <option value="tencentcloud">Tencent Cloud</option>
                <option value="other">Other</option>
              </select>
            </Field>
            <Field label="Access Key ID *">
              <Input
                value={form.accessKeyId}
                onChange={(e) => set({ accessKeyId: e.target.value })}
                required
                placeholder="AKIA..."
                className="font-mono text-sm"
              />
            </Field>
            <Field label={`Secret ${isEditing ? SECRET_HINT_KEEP : '*'}`}>
              <textarea
                value={form.secret}
                onChange={(e) => set({ secret: e.target.value })}
                className={textAreaClass + ' font-mono'}
                rows={3}
                required={!isEditing}
              />
            </Field>
            <Field label="Region">
              <Input value={form.region} onChange={(e) => set({ region: e.target.value })} placeholder="cn-north-1" />
            </Field>
          </>
        );
      case 'license':
        return (
          <>
            <Field label="Software Name *">
              <Input value={form.softwareName} onChange={(e) => set({ softwareName: e.target.value })} required placeholder="JetBrains All Products" />
            </Field>
            <Field label={`License Key ${isEditing ? SECRET_HINT_KEEP : '*'}`}>
              <textarea
                value={form.licenseKey}
                onChange={(e) => set({ licenseKey: e.target.value })}
                className={textAreaClass + ' font-mono'}
                rows={3}
                required={!isEditing}
              />
            </Field>
            <Field label="Bound Email">
              <Input type="email" value={form.boundEmail} onChange={(e) => set({ boundEmail: e.target.value })} />
            </Field>
            <Field label="Expiry Date">
              <Input type="date" value={form.expiryDate} onChange={(e) => set({ expiryDate: e.target.value })} />
            </Field>
          </>
        );
      case 'smtp':
        return (
          <>
            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-2">
                <Field label="SMTP Host *">
                  <Input value={form.host} onChange={(e) => set({ host: e.target.value })} required placeholder="smtp.example.com" />
                </Field>
              </div>
              <Field label="Port">
                <Input type="number" value={form.port} onChange={(e) => set({ port: e.target.value })} placeholder="587" />
              </Field>
            </div>
            <Field label="Encryption">
              <select value={form.encryption} onChange={(e) => set({ encryption: e.target.value })} className={selectClass}>
                <option value="">Select...</option>
                <option value="starttls">STARTTLS</option>
                <option value="ssl">SSL/TLS</option>
                <option value="none">None</option>
              </select>
            </Field>
            <Field label="Username">
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} />
            </Field>
            <Field label={`Password ${isEditing ? SECRET_HINT_KEEP : '*'}`}>
              <Input type="password" value={form.password} onChange={(e) => set({ password: e.target.value })} required={!isEditing} />
            </Field>
            <Field label="From Address">
              <Input type="email" value={form.fromAddress} onChange={(e) => set({ fromAddress: e.target.value })} placeholder="noreply@example.com" />
            </Field>
          </>
        );
    }
  };

  return (
    <Dialog open={isOpen} onClose={handleClose} title={isEditing ? 'Edit Item' : 'Add Item'}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {!isEditing && (
          <Field label="Type">
            <ItemTypeSelect value={currentType} onChange={setNewItemType} />
          </Field>
        )}
        <Field label="Group">
          <select
            value={form.groupId}
            onChange={(e) => set({ groupId: e.target.value })}
            className={selectClass}
            disabled={isEditing}
          >
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.icon} {g.name}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Title *">
          <Input value={form.title} onChange={(e) => set({ title: e.target.value })} required />
        </Field>
        {renderFields()}
        <Field label="Notes">
          <textarea value={form.notes} onChange={(e) => set({ notes: e.target.value })} className={textAreaClass} rows={3} />
        </Field>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit">{isEditing ? 'Save' : 'Add'}</Button>
        </div>
      </form>
    </Dialog>
  );
}
