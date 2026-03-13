import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { PasswordStrength } from '@/components/ui/password-strength';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import type { EnvVarPair } from '@/types';

export function AddEditItemModal() {
  const isOpen = useUIStore((s) => s.isAddItemModalOpen || s.editingItemId !== null);
  const editingItemId = useUIStore((s) => s.editingItemId);
  const closeAddItemModal = useUIStore((s) => s.closeAddItemModal);
  const closeEditItemModal = useUIStore((s) => s.closeEditItemModal);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const groups = useVaultStore((s) => s.groups);
  const createItem = useVaultStore((s) => s.createItem);
  const updateItem = useVaultStore((s) => s.updateItem);
  const createApiKeyItem = useVaultStore((s) => s.createApiKeyItem);
  const updateApiKeyItem = useVaultStore((s) => s.updateApiKeyItem);
  const createEnvVarItem = useVaultStore((s) => s.createEnvVarItem);
  const updateEnvVarItem = useVaultStore((s) => s.updateEnvVarItem);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);
  const newItemType = useUIStore((s) => s.newItemType);
  const setNewItemType = useUIStore((s) => s.setNewItemType);

  const [accountForm, setAccountForm] = useState({
    groupId: '',
    title: '',
    username: '',
    password: '',
    website: '',
    notes: '',
  });

  const [apiKeyForm, setApiKeyForm] = useState({
    groupId: '',
    title: '',
    keyName: '',
    keyValue: '',
    endpoint: '',
    authMethod: '',
    notes: '',
  });

  const [envVarForm, setEnvVarForm] = useState({
    groupId: '',
    title: '',
    variables: [{ key: '', value: '' }] as EnvVarPair[],
    notes: '',
  });

  const editingType = selectedItem?.type || 'account';
  const isEditing = editingItemId !== null;
  const currentType = isEditing ? editingType : newItemType;

  useEffect(() => {
    if (isEditing && selectedItem) {
      if (selectedItem.type === 'account') {
        setAccountForm({
          groupId: selectedItem.group_id,
          title: selectedItem.title,
          username: selectedItem.username,
          password: '',
          website: selectedItem.website || '',
          notes: selectedItem.notes || '',
        });
      } else if (selectedItem.type === 'api_key') {
        setApiKeyForm({
          groupId: selectedItem.group_id,
          title: selectedItem.title,
          keyName: selectedItem.key_name,
          keyValue: '',
          endpoint: selectedItem.endpoint || '',
          authMethod: selectedItem.auth_method || '',
          notes: selectedItem.notes || '',
        });
      } else if (selectedItem.type === 'env_var') {
        setEnvVarForm({
          groupId: selectedItem.group_id,
          title: selectedItem.title,
          variables: selectedItem.variables.length > 0 ? selectedItem.variables : [{ key: '', value: '' }],
          notes: selectedItem.notes || '',
        });
      }
    } else {
      const defaultGroupId = selectedGroupId || groups[0]?.id || '';
      setAccountForm({
        groupId: defaultGroupId,
        title: '',
        username: '',
        password: '',
        website: '',
        notes: '',
      });
      setApiKeyForm({
        groupId: defaultGroupId,
        title: '',
        keyName: '',
        keyValue: '',
        endpoint: '',
        authMethod: '',
        notes: '',
      });
      setEnvVarForm({
        groupId: defaultGroupId,
        title: '',
        variables: [{ key: '', value: '' }],
        notes: '',
      });
    }
  }, [isEditing, selectedItem, selectedGroupId, groups]);

  const handleClose = () => {
    closeAddItemModal();
    closeEditItemModal();
    setNewItemType('account');
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log('[DEBUG] handleSubmit: currentType =', currentType, 'isEditing =', isEditing);
    
    if (currentType === 'account') {
      console.log('[DEBUG] handleSubmit account: groupId =', accountForm.groupId);
      if (isEditing) {
        await updateItem(editingItemId!, {
          title: accountForm.title,
          username: accountForm.username,
          password: accountForm.password || undefined,
          website: accountForm.website || undefined,
          notes: accountForm.notes || undefined,
        });
      } else {
        await createItem({
          groupId: accountForm.groupId,
          title: accountForm.title,
          username: accountForm.username,
          password: accountForm.password,
          website: accountForm.website || undefined,
          notes: accountForm.notes || undefined,
        });
      }
    } else if (currentType === 'api_key') {
      console.log('[DEBUG] handleSubmit api_key: groupId =', apiKeyForm.groupId, 'groups =', groups);
      if (!apiKeyForm.groupId) {
        console.error('[DEBUG] handleSubmit api_key: groupId is empty!');
        return;
      }
      if (isEditing) {
        await updateApiKeyItem(editingItemId!, {
          title: apiKeyForm.title,
          keyName: apiKeyForm.keyName,
          keyValue: apiKeyForm.keyValue || undefined,
          endpoint: apiKeyForm.endpoint || undefined,
          authMethod: apiKeyForm.authMethod || undefined,
          notes: apiKeyForm.notes || undefined,
        });
      } else {
        await createApiKeyItem({
          groupId: apiKeyForm.groupId,
          title: apiKeyForm.title,
          keyName: apiKeyForm.keyName,
          keyValue: apiKeyForm.keyValue,
          endpoint: apiKeyForm.endpoint || undefined,
          authMethod: apiKeyForm.authMethod || undefined,
          notes: apiKeyForm.notes || undefined,
        });
      }
    } else if (currentType === 'env_var') {
      console.log('[DEBUG] handleSubmit env_var: groupId =', envVarForm.groupId, 'variables =', envVarForm.variables);
      if (!envVarForm.groupId) {
        console.error('[DEBUG] handleSubmit env_var: groupId is empty!');
        return;
      }
      const validVars = envVarForm.variables.filter(v => v.key.trim() && v.value.trim());
      if (isEditing) {
        await updateEnvVarItem(editingItemId!, {
          title: envVarForm.title,
          variables: validVars,
          notes: envVarForm.notes || undefined,
        });
      } else {
        await createEnvVarItem({
          groupId: envVarForm.groupId,
          title: envVarForm.title,
          variables: validVars,
          notes: envVarForm.notes || undefined,
        });
      }
    }
    handleClose();
  };

  const addEnvVar = () => {
    setEnvVarForm({
      ...envVarForm,
      variables: [...envVarForm.variables, { key: '', value: '' }],
    });
  };

  const removeEnvVar = (index: number) => {
    if (envVarForm.variables.length > 1) {
      setEnvVarForm({
        ...envVarForm,
        variables: envVarForm.variables.filter((_, i) => i !== index),
      });
    }
  };

  const updateEnvVar = (index: number, field: 'key' | 'value', value: string) => {
    const newVars = [...envVarForm.variables];
    newVars[index] = { ...newVars[index], [field]: value };
    setEnvVarForm({ ...envVarForm, variables: newVars });
  };

  const renderAccountFields = () => (
    <>
      <div>
        <label className="mb-1 block text-sm font-medium">Username *</label>
        <Input
          value={accountForm.username}
          onChange={(e) => setAccountForm({ ...accountForm, username: e.target.value })}
          required
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">
          Password {isEditing ? '(leave blank to keep)' : '*'}
        </label>
        <Input
          type="password"
          value={accountForm.password}
          onChange={(e) => setAccountForm({ ...accountForm, password: e.target.value })}
          required={!isEditing}
        />
        <PasswordStrength password={accountForm.password} />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Website</label>
        <Input
          value={accountForm.website}
          onChange={(e) => setAccountForm({ ...accountForm, website: e.target.value })}
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Notes</label>
        <textarea
          value={accountForm.notes}
          onChange={(e) => setAccountForm({ ...accountForm, notes: e.target.value })}
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
          rows={3}
        />
      </div>
    </>
  );

  const renderApiKeyFields = () => (
    <>
      <div>
        <label className="mb-1 block text-sm font-medium">Key Name *</label>
        <Input
          value={apiKeyForm.keyName}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, keyName: e.target.value })}
          required
          placeholder="e.g., OpenAI API Key"
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">
          Key Value {isEditing ? '(leave blank to keep)' : '*'}
        </label>
        <textarea
          value={apiKeyForm.keyValue}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, keyValue: e.target.value })}
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-mono dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
          rows={4}
          required={!isEditing}
          placeholder="Paste your API key here..."
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Endpoint URL</label>
        <Input
          value={apiKeyForm.endpoint}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, endpoint: e.target.value })}
          placeholder="https://api.example.com"
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Auth Method</label>
        <select
          value={apiKeyForm.authMethod}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, authMethod: e.target.value })}
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
        >
          <option value="">Select...</option>
          <option value="bearer">Bearer Token</option>
          <option value="api_key">API Key Header</option>
          <option value="basic">Basic Auth</option>
          <option value="custom">Custom</option>
        </select>
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Notes</label>
        <textarea
          value={apiKeyForm.notes}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, notes: e.target.value })}
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
          rows={3}
        />
      </div>
    </>
  );

  const renderEnvVarFields = () => (
    <>
      <div>
        <div className="mb-2 flex items-center justify-between">
          <label className="text-sm font-medium">Variables *</label>
          <Button type="button" variant="outline" size="sm" onClick={addEnvVar}>
            + Add Variable
          </Button>
        </div>
        <div className="space-y-2">
          {envVarForm.variables.map((v, i) => (
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
              {envVarForm.variables.length > 1 && (
                <Button type="button" variant="ghost" size="sm" onClick={() => removeEnvVar(i)}>
                  ×
                </Button>
              )}
            </div>
          ))}
        </div>
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Notes</label>
        <textarea
          value={envVarForm.notes}
          onChange={(e) => setEnvVarForm({ ...envVarForm, notes: e.target.value })}
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
          rows={3}
        />
      </div>
    </>
  );

  const getGroupId = () => {
    if (currentType === 'account') return accountForm.groupId;
    if (currentType === 'api_key') return apiKeyForm.groupId;
    return envVarForm.groupId;
  };

  const setGroupId = (id: string) => {
    if (currentType === 'account') {
      setAccountForm({ ...accountForm, groupId: id });
    } else if (currentType === 'api_key') {
      setApiKeyForm({ ...apiKeyForm, groupId: id });
    } else {
      setEnvVarForm({ ...envVarForm, groupId: id });
    }
  };

  const getTitle = () => {
    if (currentType === 'account') return accountForm.title;
    if (currentType === 'api_key') return apiKeyForm.title;
    return envVarForm.title;
  };

  const setTitle = (title: string) => {
    if (currentType === 'account') {
      setAccountForm({ ...accountForm, title });
    } else if (currentType === 'api_key') {
      setApiKeyForm({ ...apiKeyForm, title });
    } else {
      setEnvVarForm({ ...envVarForm, title });
    }
  };

  return (
    <Dialog open={isOpen} onClose={handleClose} title={isEditing ? 'Edit Item' : 'Add Item'}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {!isEditing && (
          <div>
            <label className="mb-1 block text-sm font-medium">Type</label>
            <select
              value={currentType}
              onChange={(e) => setNewItemType(e.target.value as 'account' | 'api_key' | 'env_var')}
              className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
            >
              <option value="account">Account</option>
              <option value="api_key">API Key</option>
              <option value="env_var">Environment Variables</option>
            </select>
          </div>
        )}
        <div>
          <label className="mb-1 block text-sm font-medium">Group</label>
          <select
            value={getGroupId()}
            onChange={(e) => setGroupId(e.target.value)}
            className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
            disabled={isEditing}
          >
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.icon} {g.name}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Title *</label>
          <Input
            value={getTitle()}
            onChange={(e) => setTitle(e.target.value)}
            required
          />
        </div>
        {currentType === 'account' && renderAccountFields()}
        {currentType === 'api_key' && renderApiKeyFields()}
        {currentType === 'env_var' && renderEnvVarFields()}
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