import { useEffect } from 'react';
import { useVaultStore } from '@/stores/vault';
import { useActivityTracker } from '@/hooks/useActivityTracker';
import { SetupScreen } from '@/components/unlock/SetupScreen';
import { UnlockScreen } from '@/components/unlock/UnlockScreen';
import { Sidebar } from '@/components/layout/Sidebar';
import { ItemList } from '@/components/layout/ItemList';
import { DetailPanel } from '@/components/layout/DetailPanel';
import { AddEditItemModal } from '@/components/modals/AddEditItemModal';
import { AddGroupModal } from '@/components/modals/AddGroupModal';
import { EditGroupModal } from '@/components/modals/EditGroupModal';
import { DeleteConfirmModal } from '@/components/modals/DeleteConfirmModal';

function MainApp() {
  useActivityTracker();
  const loadItems = useVaultStore((s) => s.loadItems);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  return (
    <div className="flex h-screen">
      <Sidebar />
      <ItemList />
      <DetailPanel />
      <AddEditItemModal />
      <AddGroupModal />
      <EditGroupModal />
      <DeleteConfirmModal />
    </div>
  );
}

function App() {
  const isInitialized = useVaultStore((s) => s.isInitialized);
  const isUnlocked = useVaultStore((s) => s.isUnlocked);
  const checkInitialized = useVaultStore((s) => s.checkInitialized);

  useEffect(() => {
    checkInitialized();
  }, [checkInitialized]);

  if (isInitialized === null) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-100">
        <p>Loading...</p>
      </div>
    );
  }

  if (!isInitialized) {
    return <SetupScreen />;
  }

  if (!isUnlocked) {
    return <UnlockScreen />;
  }

  return <MainApp />;
}

export default App;