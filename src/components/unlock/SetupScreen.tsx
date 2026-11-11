import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useVaultStore } from '@/stores/vault';
import { useI18n } from '@/i18n';

const MIN_LENGTH = 8;

export function SetupScreen() {
  const { t } = useI18n();
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState('');
  const initialize = useVaultStore((s) => s.initialize);
  const isLoading = useVaultStore((s) => s.isLoading);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (password.length < MIN_LENGTH) {
      setError(t('setup.passwordTooShort', { count: MIN_LENGTH }));
      return;
    }
    if (password !== confirm) {
      setError(t('setup.passwordsMismatch'));
      return;
    }

    await initialize(password);
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-100 dark:bg-gray-900">
      <div className="w-full max-w-sm rounded-lg bg-white p-8 shadow-lg dark:bg-gray-800">
        <h1 className="mb-2 text-center text-2xl font-bold dark:text-gray-100">SecKeyBox</h1>
        <p className="mb-6 text-center text-gray-500 dark:text-gray-300">{t('setup.subtitle')}</p>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="mb-1 block text-sm font-medium dark:text-gray-300">{t('setup.masterPassword')}</label>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t('setup.masterPasswordPlaceholder')}
              autoFocus
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium dark:text-gray-300">{t('setup.confirmPassword')}</label>
            <Input
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              placeholder={t('setup.confirmPasswordPlaceholder')}
            />
          </div>
          {error && <p className="text-sm text-red-500 dark:text-red-400">{error}</p>}
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? t('setup.creating') : t('setup.createVault')}
          </Button>
        </form>
      </div>
    </div>
  );
}
