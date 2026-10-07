import { useEffect } from 'react';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { useActivityTracker } from '@/hooks/useActivityTracker';
import { SetupScreen } from '@/components/unlock/SetupScreen';
import { UnlockScreen } from '@/components/unlock/UnlockScreen';
import { Sidebar } from '@/components/layout/Sidebar';
import { ItemList } from '@/components/layout/ItemList';
import { DetailPanel } from '@/components/layout/DetailPanel';
import { SecurityPanel } from '@/components/security/SecurityPanel';
import { AddEditItemModal } from '@/components/modals/AddEditItemModal';
import { AddGroupModal } from '@/components/modals/AddGroupModal';
import { EditGroupModal } from '@/components/modals/EditGroupModal';
import { DeleteConfirmModal } from '@/components/modals/DeleteConfirmModal';

function MainApp() {
  useActivityTracker();
  const loadItems = useVaultStore((s) => s.loadItems);
  const isSecurityPanelOpen = useUIStore((s) => s.isSecurityPanelOpen);
  const closeSecurityPanel = useUIStore((s) => s.closeSecurityPanel);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  return (
    <div className="flex h-screen">
      <Sidebar />
      {isSecurityPanelOpen ? (
        <SecurityPanel onClose={closeSecurityPanel} />
      ) : (
        <>
          <ItemList />
          <DetailPanel />
        </>
      )}
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
    console.log('App: calling checkInitialized');
    checkInitialized()
      .then(() => console.log('App: checkInitialized done, isInitialized =', useVaultStore.getState().isInitialized))
      .catch((e) => console.error('App: checkInitialized error', e));
  }, [checkInitialized]);

  console.log('App render:', { isInitialized, isUnlocked });

  if (isInitialized === null) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-100 dark:bg-gray-900">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary-500 border-t-transparent mx-auto mb-4"></div>
          <p className="text-gray-500 dark:text-gray-400">Loading...</p>
        </div>
      </div>
    );
  }

  if (!isInitialized) {
    console.log('App: showing SetupScreen');
    return <SetupScreen />;
  }

  if (!isUnlocked) {
    console.log('App: showing UnlockScreen');
    return <UnlockScreen />;
  }

  return <MainApp />;
}

export default App;