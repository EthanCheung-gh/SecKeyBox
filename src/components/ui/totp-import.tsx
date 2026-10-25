import { useRef, useState } from 'react';
import { QrCode, ClipboardPaste, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { decodeQrFromImageFile, parseOtpauthUri, type OtpauthResult } from '@/lib/otpauth';

interface TotpImportProps {
  onImport: (result: OtpauthResult) => void;
}

/**
 * Import a TOTP secret from an `otpauth://` URI or a QR screenshot.
 * Both paths land in the same parser, and the caller fills the secret field.
 */
export function TotpImport({ onImport }: TotpImportProps) {
  const [isPasteOpen, setIsPasteOpen] = useState(false);
  const [uri, setUri] = useState('');
  const [error, setError] = useState<string | null>(null);
  const imageInputRef = useRef<HTMLInputElement>(null);

  const apply = (result: OtpauthResult) => {
    if (result.digits !== undefined && result.digits !== 6) {
      setError(`Only 6-digit TOTP is supported; this entry uses ${result.digits} digits`);
      return;
    }
    if (result.period !== undefined && result.period !== 30) {
      setError(`Only 30-second TOTP is supported; this entry uses ${result.period}s`);
      return;
    }
    onImport(result);
    setIsPasteOpen(false);
    setUri('');
    setError(null);
  };

  const handlePaste = () => {
    try {
      apply(parseOtpauthUri(uri));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleImage = async (file: File | undefined) => {
    if (!file) return;
    try {
      const decoded = await decodeQrFromImageFile(file);
      apply(parseOtpauthUri(decoded));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      if (imageInputRef.current) imageInputRef.current.value = '';
    }
  };

  return (
    <div className="mt-1">
      <div className="flex gap-2">
        <Button type="button" variant="outline" size="sm" onClick={() => setIsPasteOpen(true)}>
          <ClipboardPaste className="mr-1 h-3.5 w-3.5" /> Paste URI
        </Button>
        <Button type="button" variant="outline" size="sm" onClick={() => imageInputRef.current?.click()}>
          <QrCode className="mr-1 h-3.5 w-3.5" /> Scan image
        </Button>
        <input
          ref={imageInputRef}
          type="file"
          accept="image/*"
          className="hidden"
          onChange={(e) => void handleImage(e.target.files?.[0])}
        />
      </div>

      {error && (
        <p className="mt-1 flex items-start gap-1 text-xs text-red-500">
          <AlertCircle className="mt-0.5 h-3 w-3 shrink-0" />
          {error}
        </p>
      )}

      <Dialog open={isPasteOpen} onClose={() => setIsPasteOpen(false)} title="Import TOTP URI">
        <div className="space-y-4">
          <div>
            <label className="mb-1 block text-sm font-medium">otpauth:// URI or base32 secret</label>
            <Input
              value={uri}
              onChange={(e) => setUri(e.target.value)}
              placeholder="otpauth://totp/GitHub:me?secret=JBSWY3DPEHPK3PXP&issuer=GitHub"
              className="font-mono text-sm"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  handlePaste();
                }
              }}
            />
          </div>
          {error && (
            <p className="text-xs text-red-500">{error}</p>
          )}
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setIsPasteOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handlePaste}>Import</Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}
