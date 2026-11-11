import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useVaultStore } from '@/stores/vault';
import { useI18n } from '@/i18n';

export function UnlockScreen() {
  const { t } = useI18n();
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const unlock = useVaultStore((s) => s.unlock);
  const isLoading = useVaultStore((s) => s.isLoading);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    await unlock(password);
    const storeError = useVaultStore.getState().error;
    if (storeError) {
      setError(t('unlock.invalidPassword'));
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-100 dark:bg-gray-900">
      <div className="w-full max-w-sm rounded-lg bg-white p-8 shadow-lg dark:bg-gray-800">
        <h1 className="mb-2 text-center text-2xl font-bold dark:text-gray-100">SecKeyBox</h1>
        <p className="mb-6 text-center text-gray-500 dark:text-gray-300">{t('unlock.subtitle')}</p>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t('unlock.placeholder')}
              autoFocus
            />
          </div>
          {error && <p className="text-sm text-red-500 dark:text-red-400">{error}</p>}
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? t('unlock.unlocking') : t('unlock.unlock')}
          </Button>
        </form>
      </div>
    </div>
  );
}
