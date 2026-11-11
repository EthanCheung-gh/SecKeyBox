import { useCallback, useEffect, useState } from 'react';
import { RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { getTotpCode } from '@/lib/tauri';
import { copySecret } from '@/lib/clipboard';
import { useI18n } from '@/i18n';

const PERIOD = 30;

// 详情面板内的 TOTP 实时验证码 + 倒计时环。code 与倒计时都来自后端
// （secret 只以密文落库），前端只负责定时轮询与渲染。
export function TotpCode({ itemId, secret }: { itemId: string; secret?: string }) {
  const { t } = useI18n();
  const [code, setCode] = useState<string | null>(null);
  const [remaining, setRemaining] = useState(PERIOD);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const r = await getTotpCode(itemId);
      setCode(r.code);
      setRemaining(r.seconds_remaining);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [itemId]);

  useEffect(() => {
    if (!secret) return;
    void refresh();
    const timer = setInterval(() => {
      setRemaining((r) => {
        if (r <= 1) {
          void refresh();
          return PERIOD;
        }
        return r - 1;
      });
    }, 1000);
    return () => clearInterval(timer);
  }, [secret, refresh]);

  if (!secret) return null;

  const progress = (remaining / PERIOD) * 100;

  const copy = async () => {
    if (code) {
      await copySecret(code).catch(() => undefined);
    }
  };

  return (
    <div>
      <label className="mb-1 block text-sm font-medium text-gray-500 dark:text-gray-300">
        {t('totp.title')}
      </label>
      <div className="flex items-center gap-3">
        <span className="font-mono text-2xl font-semibold tracking-widest dark:text-gray-100">
          {code ?? (error ? '------' : '......')}
        </span>
        <div className="relative h-8 w-8">
          <svg className="h-8 w-8 -rotate-90" viewBox="0 0 36 36">
            <circle cx="18" cy="18" r="16" fill="none" className="stroke-gray-200 dark:stroke-gray-700" strokeWidth="3" />
            <circle
              cx="18"
              cy="18"
              r="16"
              fill="none"
              className={remaining <= 5 ? 'stroke-red-500' : 'stroke-primary-500'}
              strokeWidth="3"
              strokeDasharray={`${(progress / 100) * 100.5} 100.5`}
              strokeLinecap="round"
            />
          </svg>
          <span className="absolute inset-0 flex items-center justify-center text-[10px] text-gray-500 dark:text-gray-300">
            {remaining}
          </span>
        </div>
        <Button variant="ghost" size="sm" onClick={copy} title={t('totp.copyCode')}>
          <span className="text-xs">{t('totp.copy')}</span>
        </Button>
        <Button variant="ghost" size="sm" onClick={refresh} title={t('totp.refreshNow')}>
          <RefreshCw className="h-3.5 w-3.5" />
        </Button>
      </div>
      {error && <p className="mt-1 text-xs text-red-500">{t('totp.error', { error })}</p>}
    </div>
  );
}
