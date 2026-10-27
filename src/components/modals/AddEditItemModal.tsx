import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { PasswordStrength } from '@/components/ui/password-strength';
import { PasswordGenerator } from '@/components/ui/password-generator';
import { TotpImport } from '@/components/ui/totp-import';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import type { EnvVarPair } from '@/types';
import { useI18n, groupDisplayName } from '@/i18n';
import { listCustomFields, saveCustomFields, getItemTags, setItemTags } from '@/lib/tauri';

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
  // account: 两步验证（base32 secret；编辑时空串表示清除）
  totpSecret: '',
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
}) => {
  const { t } = useI18n();
  return (
  <select
    value={value}
    onChange={(e) => onChange(e.target.value as Parameters<typeof onChange>[0])}
    className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
  >
    <option value="account">{t('type.account')}</option>
    <option value="api_key">{t('type.api_key')}</option>
    <option value="env_var">{t('type.env_var')}</option>
    <option value="database">{t('type.database')}</option>
    <option value="ssh">{t('type.ssh')}</option>
    <option value="cloud">{t('type.cloud')}</option>
    <option value="license">{t('type.license')}</option>
    <option value="smtp">{t('type.smtp')}</option>
  </select>
  );
};

const Field = ({ label, children }: { label: string; children: React.ReactNode }) => (
  <div>
    <label className="mb-1 block text-sm font-medium">{label}</label>
    {children}
  </div>
);

const textAreaClass =
  'w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100';
const selectClass = textAreaClass;



export function AddEditItemModal() {
  const { t } = useI18n();
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
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [customFields, setCustomFields] = useState<{ key: string; value: string }[]>([]);
  const [selectedTagIds, setSelectedTagIds] = useState<string[]>([]);
  const tags = useVaultStore((s) => s.tags);
  const set = (patch: Partial<ItemForm>) => setForm({ ...form, ...patch });

  const isEditing = editingItemId !== null;
  const editingType = selectedItem?.type || 'account';
  const currentType = isEditing ? editingType : newItemType;

  useEffect(() => {
    if (!isEditing || !editingItemId) {
      setCustomFields([]);
      setSelectedTagIds([]);
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        const [fields, tagIds] = await Promise.all([
          listCustomFields(editingItemId),
          getItemTags(editingItemId),
        ]);
        if (cancelled) return;
        setCustomFields(fields.map((field) => ({ key: field.key, value: field.value })));
        setSelectedTagIds(tagIds);
      } catch {
        if (!cancelled) {
          setCustomFields([]);
          setSelectedTagIds([]);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [isEditing, editingItemId]);

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
        totpSecret: '',
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
          setForm({
            ...base,
            username: selectedItem.username,
            website: selectedItem.website || '',
            totpSecret: selectedItem.totp_secret ?? '',
          });
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
    setSubmitError(null);
    const id = editingItemId;
    const opt = (v: string) => (v === '' ? undefined : v);
    const secret = (v: string) => (v === '' ? undefined : v); // 空 = 保持原值
    const num = (v: string) => (v === '' ? undefined : Number(v));
    // Id of the item after save (existing id when editing, new id on create).
    let savedId: string | undefined = isEditing ? id ?? undefined : undefined;

    try {
      switch (currentType) {
      case 'account':
        if (isEditing) {
          await vault.updateItem(id!, {
            title: form.title,
            username: form.username,
            password: secret(form.password),
            website: opt(form.website),
            notes: opt(form.notes),
            // None=keep（不改 TOTP）不可表达：空串即清除，有值即设置
            totpSecret: form.totpSecret,
          });
        } else {
          savedId = await vault.createItem({
            groupId: form.groupId,
            title: form.title,
            username: form.username,
            password: form.password,
            website: opt(form.website),
            notes: opt(form.notes),
            totpSecret: form.totpSecret || undefined,
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
          savedId = await vault.createApiKeyItem({
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
          savedId = await vault.createEnvVarItem({
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
          savedId = await vault.createDatabaseItem({
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
          savedId = await vault.createSshItem({
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
          savedId = await vault.createCloudItem({
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
          savedId = await vault.createLicenseItem({
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
          savedId = await vault.createSmtpItem({
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
      if (savedId) {
        await saveCustomFields(
          savedId,
          customFields
            .map((field) => ({ key: field.key.trim(), value: field.value }))
            .filter((field) => field.key !== '')
        );
        await setItemTags(savedId, selectedTagIds);
      }
    } catch (err) {
      // store 已写入全局 error；这里在弹窗内直接展示，且不关闭弹窗，
      // 避免出现“提交失败却看起来成功”的无感知体验。
      // Tauri 拒绝的是一个普通对象（如 { Validation: "..." }），
      // String(err) 会得到 "[object Object]"，所以对象要序列化展示。
      setSubmitError(
        err instanceof Error
          ? err.message
          : typeof err === 'string'
            ? err
            : JSON.stringify(err)
      );
      return;
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
            <Field label={t('form.username')}>
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} required />
            </Field>
            <Field label={`Password ${isEditing ? t('form.keepSecretHint') : '*'}`}>
              <Input
                type="password"
                value={form.password}
                onChange={(e) => set({ password: e.target.value })}
                required={!isEditing}
              />
              <div className="mt-1">
                <PasswordGenerator onUse={(pw) => set({ password: pw })} />
              </div>
              <PasswordStrength password={form.password} />
            </Field>
            <Field label={t('form.totpSecret')}>
              <Input
                value={form.totpSecret}
                onChange={(e) => set({ totpSecret: e.target.value })}
                placeholder={t('form.totpPlaceholder')}
                className="font-mono text-sm"
              />
              <TotpImport
                onImport={(result) =>
                  set({
                    totpSecret: result.secret,
                    // Fill an empty title from the QR metadata, but never
                    // overwrite something the user already typed.
                    title: form.title || result.issuer || result.account || '',
                  })
                }
              />
            </Field>
            <Field label={t('form.website')}>
              <Input value={form.website} onChange={(e) => set({ website: e.target.value })} />
            </Field>
          </>
        );
      case 'api_key':
        return (
          <>
            <Field label={t('form.keyName')}>
              <Input
                value={form.keyName}
                onChange={(e) => set({ keyName: e.target.value })}
                required
                placeholder="e.g., OpenAI API Key"
              />
            </Field>
            <Field label={`Key Value ${isEditing ? t('form.keepSecretHint') : '*'}`}>
              <textarea
                value={form.keyValue}
                onChange={(e) => set({ keyValue: e.target.value })}
                className={textAreaClass + ' font-mono'}
                rows={4}
                required={!isEditing}
                placeholder="Paste your API key here..."
              />
            </Field>
            <Field label={t('detail.endpoint')}>
              <Input
                value={form.endpoint}
                onChange={(e) => set({ endpoint: e.target.value })}
                placeholder="https://api.example.com"
              />
            </Field>
            <Field label={t('form.authMethod')}>
              <select
                value={form.authMethod}
                onChange={(e) => set({ authMethod: e.target.value })}
                className={selectClass}
              >
                <option value="">{t('common.select')}</option>
                <option value="bearer">{t('form.authBearer')}</option>
                <option value="api_key">{t('form.authApiKeyHeader')}</option>
                <option value="basic">{t('form.authBasic')}</option>
                <option value="custom">{t('form.authCustom')}</option>
              </select>
            </Field>
          </>
        );
      case 'env_var':
        return (
          <Field label={`${t('form.variables')} *`}>
            <div className="mb-2 flex justify-end">
              <Button type="button" variant="outline" size="sm" onClick={addEnvVar}>
                + {t('form.addVariable')}
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
            <Field label={`${t('form.dbType')} *`}>
              <select value={form.dbType} onChange={(e) => set({ dbType: e.target.value })} className={selectClass} required>
                <option value="">{t('common.select')}</option>
                <option value="mysql">MySQL / MariaDB</option>
                <option value="postgresql">PostgreSQL</option>
                <option value="redis">Redis</option>
                <option value="mongodb">MongoDB</option>
                <option value="sqlserver">SQL Server</option>
                <option value="sqlite">SQLite</option>
                <option value="other">{t('common.other')}</option>
              </select>
            </Field>
            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-2">
                <Field label={`${t('detail.host')} *`}>
                  <Input value={form.host} onChange={(e) => set({ host: e.target.value })} required placeholder="db.example.com" />
                </Field>
              </div>
              <Field label={t('detail.port')}>
                <Input type="number" value={form.port} onChange={(e) => set({ port: e.target.value })} placeholder="3306" />
              </Field>
            </div>
            <Field label={t('form.databaseName')}>
              <Input value={form.databaseName} onChange={(e) => set({ databaseName: e.target.value })} />
            </Field>
            <Field label={t('form.username').replace(' *', '')}>
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} />
            </Field>
            <Field label={`Password ${isEditing ? t('form.keepSecretHint') : ''}`}>
              <Input
                type="password"
                value={form.password}
                onChange={(e) => set({ password: e.target.value })}
              />
            </Field>
            <Field label={t('form.connectionUrl')}>
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
                <Field label={`${t('detail.host')} *`}>
                  <Input value={form.host} onChange={(e) => set({ host: e.target.value })} required placeholder="server.example.com" />
                </Field>
              </div>
              <Field label={t('detail.port')}>
                <Input type="number" value={form.port} onChange={(e) => set({ port: e.target.value })} placeholder="22" />
              </Field>
            </div>
            <Field label={t('form.username')}>
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} required placeholder="root" />
            </Field>
            <Field label={`${t('form.password')} ${isEditing ? t('form.keepSecretHint') : t('form.leaveBlankIfKeyAuth')}`}>
              <Input type="password" value={form.password} onChange={(e) => set({ password: e.target.value })} />
            </Field>
            <Field label={t('form.keyPath')}>
              <Input
                value={form.keyPath}
                onChange={(e) => set({ keyPath: e.target.value })}
                placeholder="~/.ssh/id_ed25519"
                className="font-mono text-sm"
              />
            </Field>
            <Field label={`Key Passphrase ${isEditing ? t('form.keepSecretHint') : ''}`}>
              <Input type="password" value={form.passphrase} onChange={(e) => set({ passphrase: e.target.value })} />
            </Field>
          </>
        );
      case 'cloud':
        return (
          <>
            <Field label={`${t('form.provider')} *`}>
              <select value={form.provider} onChange={(e) => set({ provider: e.target.value })} className={selectClass} required>
                <option value="">{t('common.select')}</option>
                <option value="aws">AWS</option>
                <option value="gcp">Google Cloud</option>
                <option value="azure">Azure</option>
                <option value="alicloud">Alibaba Cloud</option>
                <option value="tencentcloud">Tencent Cloud</option>
                <option value="other">{t('common.other')}</option>
              </select>
            </Field>
            <Field label={t('form.accessKeyId')}>
              <Input
                value={form.accessKeyId}
                onChange={(e) => set({ accessKeyId: e.target.value })}
                required
                placeholder="AKIA..."
                className="font-mono text-sm"
              />
            </Field>
            <Field label={`Secret ${isEditing ? t('form.keepSecretHint') : '*'}`}>
              <textarea
                value={form.secret}
                onChange={(e) => set({ secret: e.target.value })}
                className={textAreaClass + ' font-mono'}
                rows={3}
                required={!isEditing}
              />
            </Field>
            <Field label={t('form.region')}>
              <Input value={form.region} onChange={(e) => set({ region: e.target.value })} placeholder="cn-north-1" />
            </Field>
          </>
        );
      case 'license':
        return (
          <>
            <Field label={t('form.softwareName')}>
              <Input value={form.softwareName} onChange={(e) => set({ softwareName: e.target.value })} required placeholder="JetBrains All Products" />
            </Field>
            <Field label={`License Key ${isEditing ? t('form.keepSecretHint') : '*'}`}>
              <textarea
                value={form.licenseKey}
                onChange={(e) => set({ licenseKey: e.target.value })}
                className={textAreaClass + ' font-mono'}
                rows={3}
                required={!isEditing}
              />
            </Field>
            <Field label={t('form.boundEmail')}>
              <Input type="email" value={form.boundEmail} onChange={(e) => set({ boundEmail: e.target.value })} />
            </Field>
            <Field label={t('form.expiryDate')}>
              <Input
                type="date"
                value={form.expiryDate}
                onChange={(e) => {
                  set({ expiryDate: e.target.value });
                  // WebKitGTK (Linux) 的日历弹层在选中日期后不会自动收起，
                  // 主动移开焦点以关闭它。
                  e.target.blur();
                }}
              />
            </Field>
          </>
        );
      case 'smtp':
        return (
          <>
            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-2">
                <Field label={t('form.smtpHost')}>
                  <Input value={form.host} onChange={(e) => set({ host: e.target.value })} required placeholder="smtp.example.com" />
                </Field>
              </div>
              <Field label={t('detail.port')}>
                <Input type="number" value={form.port} onChange={(e) => set({ port: e.target.value })} placeholder="587" />
              </Field>
            </div>
            <Field label={t('form.encryption')}>
              <select value={form.encryption} onChange={(e) => set({ encryption: e.target.value })} className={selectClass}>
                <option value="">{t('common.select')}</option>
                <option value="starttls">STARTTLS</option>
                <option value="ssl">SSL/TLS</option>
                <option value="none">None</option>
              </select>
            </Field>
            <Field label={t('form.username').replace(' *', '')}>
              <Input value={form.username} onChange={(e) => set({ username: e.target.value })} />
            </Field>
            <Field label={`Password ${isEditing ? t('form.keepSecretHint') : '*'}`}>
              <Input type="password" value={form.password} onChange={(e) => set({ password: e.target.value })} required={!isEditing} />
            </Field>
            <Field label={t('form.fromAddress')}>
              <Input type="email" value={form.fromAddress} onChange={(e) => set({ fromAddress: e.target.value })} placeholder="noreply@example.com" />
            </Field>
          </>
        );
    }
  };

  return (
    <Dialog open={isOpen} onClose={handleClose} title={isEditing ? t('form.titleEdit') : t('form.titleAdd')}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {submitError && (
          <div className="rounded-md bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
            {submitError}
          </div>
        )}
        {!isEditing && (
          <Field label={t('form.type')}>
            <ItemTypeSelect value={currentType} onChange={setNewItemType} />
          </Field>
        )}
        <Field label={t('form.group')}>
          <select
            value={form.groupId}
            onChange={(e) => set({ groupId: e.target.value })}
            className={selectClass}
            disabled={isEditing}
          >
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.icon} {groupDisplayName(g, t)}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t('form.title')}>
          <Input value={form.title} onChange={(e) => set({ title: e.target.value })} required />
        </Field>
        {renderFields()}
        <Field label={t('form.notes')}>
          <textarea value={form.notes} onChange={(e) => set({ notes: e.target.value })} className={textAreaClass} rows={3} />
        </Field>
        {tags.length > 0 && (
          <Field label={t('tags.title')}>
            <div className="flex flex-wrap gap-x-4 gap-y-1">
              {tags.map((tag) => (
                <label key={tag.id} className="flex items-center gap-1 text-sm dark:text-gray-300">
                  <input
                    type="checkbox"
                    checked={selectedTagIds.includes(tag.id)}
                    onChange={(e) =>
                      setSelectedTagIds((prev) =>
                        e.target.checked
                          ? [...prev, tag.id]
                          : prev.filter((tagId) => tagId !== tag.id)
                      )
                    }
                  />
                  {tag.name}
                </label>
              ))}
            </div>
          </Field>
        )}

        <Field label={t('customFields.title')}>
          <div className="space-y-2">
            {customFields.map((field, index) => (
              <div key={index} className="flex gap-2">
                <Input
                  value={field.key}
                  onChange={(e) =>
                    setCustomFields((prev) =>
                      prev.map((item, i) => (i === index ? { ...item, key: e.target.value } : item))
                    )
                  }
                  placeholder={t('customFields.key')}
                  className="w-40"
                />
                <Input
                  value={field.value}
                  onChange={(e) =>
                    setCustomFields((prev) =>
                      prev.map((item, i) => (i === index ? { ...item, value: e.target.value } : item))
                    )
                  }
                  placeholder={t('customFields.value')}
                  className="flex-1"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => setCustomFields((prev) => prev.filter((_, i) => i !== index))}
                  title={t('customFields.remove')}
                >
                  ×
                </Button>
              </div>
            ))}
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setCustomFields((prev) => [...prev, { key: '', value: '' }])}
            >
              + {t('customFields.add')}
            </Button>
          </div>
        </Field>

        {/* Sticky action bar: on short windows the form scrolls but Save must
            stay reachable without scrolling to the very bottom. */}
        <div className="sticky bottom-0 -mx-6 mt-2 flex justify-end gap-2 border-t border-gray-200 bg-white px-6 pb-1 pt-4 dark:border-gray-700 dark:bg-gray-800">
          <Button type="button" variant="outline" onClick={handleClose}>
            {t('common.cancel')}
          </Button>
          <Button type="submit">{isEditing ? t('form.save') : t('form.addItem')}</Button>
        </div>
      </form>
    </Dialog>
  );
}
