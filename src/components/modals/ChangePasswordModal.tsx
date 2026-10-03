import { useState } from 'react';
import { KeyRound, AlertCircle, CheckCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { PasswordStrength } from '@/components/ui/password-strength';
import { changeMasterPassword } from '@/lib/tauri';

const MIN_LENGTH = 8;

export function ChangePasswordModal() {
  const [isOpen, setIsOpen] = useState(false);
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [done, setDone] = useState(false);

  const reset = () => {
    setCurrentPassword('');
    setNewPassword('');
    setConfirmPassword('');
    setError(null);
    setDone(false);
    setIsSaving(false);
  };

  const handleClose = () => {
    setIsOpen(false);
    reset();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (newPassword.length < MIN_LENGTH) {
      setError(`New password must be at least ${MIN_LENGTH} characters`);
      return;
    }
    if (newPassword !== confirmPassword) {
      setError('New passwords do not match');
      return;
    }
    if (newPassword === currentPassword) {
      setError('New password must differ from the current one');
      return;
    }

    setIsSaving(true);
    try {
      await changeMasterPassword(currentPassword, newPassword);
      setDone(true);
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
    } catch (e) {
      // 后端对错误密码返回 InvalidPassword，其他错误原样展示
      const message = e instanceof Error ? e.message : String(e);
      setError(message.includes('InvalidPassword') ? 'Current password is incorrect' : message);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <>
      <Button
        variant="ghost"
        size="sm"
        className="w-full justify-start text-gray-500 dark:text-gray-400"
        onClick={() => setIsOpen(true)}
      >
        <KeyRound className="mr-2 h-4 w-4" /> Change Master Password
      </Button>

      <Dialog open={isOpen} onClose={handleClose} title="Change Master Password">
        {done ? (
          <div className="space-y-4">
            <div className="flex items-start gap-2 rounded-md bg-green-50 p-3 text-sm text-green-700 dark:bg-green-900/20 dark:text-green-400">
              <CheckCircle className="mt-0.5 h-4 w-4 shrink-0" />
              <span>
                Master password changed. All stored secrets were re-encrypted with the new key —
                use the new password the next time you unlock.
              </span>
            </div>
            <div className="flex justify-end">
              <Button onClick={handleClose}>Close</Button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="flex items-center gap-2 rounded-md bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
                <AlertCircle className="h-4 w-4 shrink-0" />
                {error}
              </div>
            )}

            <div>
              <label className="mb-1 block text-sm font-medium">Current Password *</label>
              <Input
                type="password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                required
                autoFocus
              />
            </div>

            <div>
              <label className="mb-1 block text-sm font-medium">New Password *</label>
              <Input
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                required
                minLength={MIN_LENGTH}
              />
              <PasswordStrength password={newPassword} />
            </div>

            <div>
              <label className="mb-1 block text-sm font-medium">Confirm New Password *</label>
              <Input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                required
                minLength={MIN_LENGTH}
              />
              {confirmPassword !== '' && confirmPassword !== newPassword && (
                <p className="mt-1 text-xs text-red-500">Passwords do not match</p>
              )}
            </div>

            <p className="text-xs text-gray-500 dark:text-gray-400">
              Every stored secret is decrypted and re-encrypted with a key derived from the new
              password. Keep it safe — there is no recovery.
            </p>

            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button type="submit" disabled={isSaving}>
                {isSaving ? 'Changing...' : 'Change Password'}
              </Button>
            </div>
          </form>
        )}
      </Dialog>
    </>
  );
}
