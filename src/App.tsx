import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { useI18n } from '@/i18n';
import { useActivityTracker } from '@/hooks/useActivityTracker';
import { SetupScreen } from '@/components/unlock/SetupScreen';
import { UnlockScreen } from '@/components/unlock/UnlockScreen';
import { Sidebar } from '@/components/layout/Sidebar';
import { ItemList } from '@/components/layout/ItemList';
import { DetailPanel } from '@/components/layout/DetailPanel';
import { SecurityPanel } from '@/components/security/SecurityPanel';
import { TrashPanel } from '@/components/trash/TrashPanel';
import { AddEditItemModal } from '@/components/modals/AddEditItemModal';
import { AddGroupModal } from '@/components/modals/AddGroupModal';
import { EditGroupModal } from '@/components/modals/EditGroupModal';
import { DeleteConfirmModal } from '@/components/modals/DeleteConfirmModal';
import { AddTagModal } from '@/components/modals/AddTagModal';

function MainApp() {
  useActivityTracker();
  const loadItems = useVaultStore((s) => s.loadItems);
  const loadTags = useVaultStore((s) => s.loadTags);
  const loadTrashCount = useVaultStore((s) => s.loadTrashCount);
  const isSecurityPanelOpen = useUIStore((s) => s.isSecurityPanelOpen);
  const isTrashOpen = useUIStore((s) => s.isTrashOpen);
  const closeSecurityPanel = useUIStore((s) => s.closeSecurityPanel);

  useEffect(() => {
    loadItems();
    void loadTags();
    void loadTrashCount();
  }, [loadItems, loadTags, loadTrashCount]);

  return (
    <div className="flex h-screen">
      <Sidebar />
      {isSecurityPanelOpen ? (
        <SecurityPanel onClose={closeSecurityPanel} />
      ) : isTrashOpen ? (
        <TrashPanel />
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
      <AddTagModal />
    </div>
  );
}

function App() {
  const { t } = useI18n();
  const isInitialized = useVaultStore((s) => s.isInitialized);
  const isUnlocked = useVaultStore((s) => s.isUnlocked);
  const checkInitialized = useVaultStore((s) => s.checkInitialized);

  useEffect(() => {
    console.log('App: calling checkInitialized');
    checkInitialized()
      .then(() => console.log('App: checkInitialized done, isInitialized =', useVaultStore.getState().isInitialized))
      .catch((e) => console.error('App: checkInitialized error', e));
  }, [checkInitialized]);

  // The Rust watchdog locks the vault (idle timeout) and notifies the UI so
  // it switches to the unlock screen right away.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>('vault-locked', () => {
      void useVaultStore.getState().lock();
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => undefined);
    return () => {
      unlisten?.();
    };
  }, []);

  // A locked (or re-unlocked) session must not restore a stale panel view:
  // the trash and security panels hold data that no longer exists.
  useEffect(() => {
    if (!isUnlocked) {
      useUIStore.getState().closeTrash();
      useUIStore.getState().closeSecurityPanel();
    }
  }, [isUnlocked]);

  console.log('App render:', { isInitialized, isUnlocked });

  if (isInitialized === null) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-100 dark:bg-gray-900">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary-500 border-t-transparent mx-auto mb-4"></div>
          <p className="text-gray-500 dark:text-gray-400">{t('app.loading')}</p>
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