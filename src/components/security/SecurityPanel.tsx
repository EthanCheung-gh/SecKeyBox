import { useCallback, useEffect, useState } from 'react';
import { ShieldAlert, ShieldCheck, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { runSecurityAudit } from '@/lib/tauri';
import { useVaultStore } from '@/stores/vault';
import type { SecurityAudit as Audit } from '@/types';

const Section = ({
  title,
  tone,
  findings,
  onJump,
}: {
  title: string;
  tone: 'warn' | 'bad';
  findings: { item_id: string; title: string; detail: string }[];
  onJump: (id: string) => void;
}) => {
  if (findings.length === 0) return null;
  const color =
    tone === 'bad'
      ? 'border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-900/20'
      : 'border-yellow-200 bg-yellow-50 dark:border-yellow-900 dark:bg-yellow-900/20';
  return (
    <div className={`rounded-md border p-3 ${color}`}>
      <h3 className="mb-2 text-sm font-medium">
        {title}
        <span className="ml-1 text-xs text-gray-500">({findings.length})</span>
      </h3>
      <ul className="space-y-1">
        {findings.map((f, i) => (
          <li key={i} className="flex items-center justify-between text-xs">
            <span className="dark:text-gray-200">
              {f.title} — <span className="text-gray-500 dark:text-gray-400">{f.detail}</span>
            </span>
            <button onClick={() => onJump(f.item_id)} className="text-primary-600 hover:underline dark:text-primary-400">
              查看
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
};

export function SecurityPanel({ onClose }: { onClose: () => void }) {
  const [audit, setAudit] = useState<Audit | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const selectItem = useVaultStore((s) => s.selectItem);

  const run = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setAudit(await runSecurityAudit());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void run();
  }, [run]);

  const jump = (id: string) => {
    void selectItem(id);
    onClose();
  };

  const total =
    (audit?.weak_passwords.length ?? 0) +
    (audit?.stale_rotations.length ?? 0) +
    (audit?.missing_2fa.length ?? 0) +
    (audit?.expiring_licenses.length ?? 0) +
    (audit?.reused_secrets.length ?? 0);

  return (
    <div className="flex h-full flex-1 flex-col bg-gray-50 p-6 dark:bg-gray-900">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          {total > 0 ? (
            <ShieldAlert className="h-5 w-5 text-yellow-500" />
          ) : (
            <ShieldCheck className="h-5 w-5 text-green-500" />
          )}
          <h2 className="text-lg font-semibold dark:text-gray-100">安全体检</h2>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={run}>
            重新体检
          </Button>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {loading && <p className="text-sm text-gray-500">分析中…</p>}
      {error && <p className="text-sm text-red-500">{error}</p>}

      {audit && !loading && (
        <div className="flex-1 space-y-3 overflow-y-auto">
          {total === 0 && (
            <p className="rounded-md border border-green-200 bg-green-50 p-4 text-sm text-green-700 dark:border-green-900 dark:bg-green-900/20 dark:text-green-400">
              未发现风险：密码强度达标、无重复密钥、2FA 覆盖率良好、无临期许可证。
            </p>
          )}
          <Section title="弱密码" tone="bad" findings={audit.weak_passwords} onJump={jump} />
          <Section title="缺少两步验证的账号" tone="warn" findings={audit.missing_2fa} onJump={jump} />
          <Section title="长期未轮换的 API Key" tone="warn" findings={audit.stale_rotations} onJump={jump} />
          <Section title="即将过期的许可证" tone="warn" findings={audit.expiring_licenses} onJump={jump} />
          {audit.reused_secrets.map((g, i) => (
            <div key={i} className="rounded-md border border-orange-200 bg-orange-50 p-3 dark:border-orange-900 dark:bg-orange-900/20">
              <h3 className="mb-2 text-sm font-medium">
                重复使用的密钥 <span className="font-mono">{g.secret_hint}</span>
                <span className="ml-1 text-xs text-gray-500">({g.items.length} 处)</span>
              </h3>
              <ul className="space-y-1">
                {g.items.map((f, j) => (
                  <li key={j} className="flex items-center justify-between text-xs">
                    <span className="dark:text-gray-200">
                      {f.title} — <span className="text-gray-500">{f.detail}</span>
                    </span>
                    <button onClick={() => jump(f.item_id)} className="text-primary-600 hover:underline dark:text-primary-400">
                      查看
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
