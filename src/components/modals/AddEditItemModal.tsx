import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

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
    }
  }, [isEditing, selectedItem, selectedGroupId, groups]);

  const handleClose = () => {
    closeAddItemModal();
    closeEditItemModal();
    setNewItemType('account');
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (currentType === 'account') {
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
    }
    handleClose();
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
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
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
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm font-mono"
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
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
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
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
          rows={3}
        />
      </div>
    </>
  );

  return (
    <Dialog open={isOpen} onClose={handleClose} title={isEditing ? 'Edit Item' : 'Add Item'}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {!isEditing && (
          <div>
            <label className="mb-1 block text-sm font-medium">Type</label>
            <select
              value={currentType}
              onChange={(e) => setNewItemType(e.target.value as 'account' | 'api_key')}
              className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
            >
              <option value="account">Account</option>
              <option value="api_key">API Key</option>
            </select>
          </div>
        )}
        <div>
          <label className="mb-1 block text-sm font-medium">Group</label>
          <select
            value={currentType === 'account' ? accountForm.groupId : apiKeyForm.groupId}
            onChange={(e) => {
              const form = { groupId: e.target.value };
              if (currentType === 'account') {
                setAccountForm({ ...accountForm, ...form });
              } else {
                setApiKeyForm({ ...apiKeyForm, ...form });
              }
            }}
            className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
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
            value={currentType === 'account' ? accountForm.title : apiKeyForm.title}
            onChange={(e) => {
              if (currentType === 'account') {
                setAccountForm({ ...accountForm, title: e.target.value });
              } else {
                setApiKeyForm({ ...apiKeyForm, title: e.target.value });
              }
            }}
            required
          />
        </div>
        {currentType === 'account' ? renderAccountFields() : renderApiKeyFields()}
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