import { useMemo, useState } from 'react';
import { RefreshCw, Check } from 'lucide-react';

const SETS = {
  lower: 'abcdefghijklmnopqrstuvwxyz',
  upper: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
  digits: '0123456789',
  symbols: '!@#$%^&*()-_=+[]{};:,.?/',
};
const AMBIGUOUS = /[O0Il1|`'"~,.;:]/g;

export interface GeneratorOptions {
  length: number;
  lower: boolean;
  upper: boolean;
  digits: boolean;
  symbols: boolean;
  avoidAmbiguous: boolean;
}

export const DEFAULT_GENERATOR_OPTIONS: GeneratorOptions = {
  length: 20,
  lower: true,
  upper: true,
  digits: true,
  symbols: true,
  avoidAmbiguous: false,
};

function activePool(opts: GeneratorOptions): { pool: string; sets: string[] } {
  const sets: string[] = [];
  if (opts.lower) sets.push(SETS.lower);
  if (opts.upper) sets.push(SETS.upper);
  if (opts.digits) sets.push(SETS.digits);
  if (opts.symbols) sets.push(SETS.symbols);
  const pool = sets
    .join('')
    .split('')
    .filter((c) => !(opts.avoidAmbiguous && AMBIGUOUS.test(c)))
    .join('');
  const cleanedSets = sets.map((s) =>
    opts.avoidAmbiguous ? s.split('').filter((c) => !AMBIGUOUS.test(c)).join('') : s,
  );
  return { pool, sets: cleanedSets.filter((s) => s.length > 0) };
}

function randomIndex(max: number): number {
  const buf = new Uint32Array(1);
  crypto.getRandomValues(buf);
  return buf[0] % max;
}

/** Cryptographically-strong random password; at least one char per enabled set. */
export function generatePassword(opts: GeneratorOptions): string {
  const { pool, sets } = activePool(opts);
  if (!pool) return '';

  const length = Math.max(opts.length, sets.length);
  const chars: string[] = [];
  // one guaranteed char from each active set
  for (const set of sets) {
    chars.push(set[randomIndex(set.length)]);
  }
  // fill the rest
  while (chars.length < length) {
    chars.push(pool[randomIndex(pool.length)]);
  }
  // Fisher-Yates shuffle so guaranteed chars are not always at the front
  for (let i = chars.length - 1; i > 0; i--) {
    const j = randomIndex(i + 1);
    [chars[i], chars[j]] = [chars[j], chars[i]];
  }
  return chars.join('');
}

export function strengthScore(password: string): number {
  let classes = 0;
  if (/[a-z]/.test(password)) classes++;
  if (/[A-Z]/.test(password)) classes++;
  if (/[0-9]/.test(password)) classes++;
  if (/[^a-zA-Z0-9]/.test(password)) classes++;
  const lengthScore = Math.min(password.length / 24, 1);
  const variety = classes / 4;
  return Math.round(lengthScore * variety * 100);
}

export function PasswordGenerator({
  onUse,
}: {
  onUse: (password: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [options, setOptions] = useState(DEFAULT_GENERATOR_OPTIONS);
  const [copied, setCopied] = useState(false);
  const password = useMemo(() => generatePassword(options), [options]);

  const strength = strengthScore(password);
  const strengthLabel = strength >= 80 ? '强' : strength >= 50 ? '中' : '弱';

  if (!open) {
    return (
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="text-xs text-primary-600 hover:underline dark:text-primary-400"
      >
        生成强密码
      </button>
    );
  }

  return (
    <div className="rounded-md border border-gray-200 p-3 dark:border-gray-600">
      <div className="mb-2 flex items-center gap-2">
        <code className="flex-1 break-all rounded bg-gray-100 px-2 py-1 font-mono text-sm dark:bg-gray-700 dark:text-gray-100">
          {password}
        </code>
        <button
          type="button"
          title="重新生成"
          onClick={() => setOptions({ ...options })}
          className="rounded p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
        >
          <RefreshCw className="h-4 w-4" />
        </button>
      </div>

      <div className="mb-2 flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
        <div className="h-1.5 flex-1 rounded-full bg-gray-200 dark:bg-gray-700">
          <div
            className={`h-full rounded-full ${strength >= 80 ? 'bg-green-500' : strength >= 50 ? 'bg-yellow-500' : 'bg-red-500'}`}
            style={{ width: `${strength}%` }}
          />
        </div>
        <span>强度: {strengthLabel}</span>
      </div>

      <label className="mb-2 flex items-center gap-2 text-xs dark:text-gray-300">
        长度 {options.length}
        <input
          type="range"
          min={8}
          max={48}
          value={options.length}
          onChange={(e) => setOptions({ ...options, length: Number(e.target.value) })}
          className="flex-1"
        />
      </label>

      <div className="mb-2 grid grid-cols-2 gap-1 text-xs dark:text-gray-300">
        {(
          [
            ['lower', '小写字母 a-z'],
            ['upper', '大写字母 A-Z'],
            ['digits', '数字 0-9'],
            ['symbols', '符号 !@#'],
          ] as const
        ).map(([key, label]) => (
          <label key={key} className="flex items-center gap-1">
            <input
              type="checkbox"
              checked={options[key]}
              onChange={(e) => setOptions({ ...options, [key]: e.target.checked })}
            />
            {label}
          </label>
        ))}
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            checked={options.avoidAmbiguous}
            onChange={(e) => setOptions({ ...options, avoidAmbiguous: e.target.checked })}
          />
          排除易混字符
        </label>
      </div>

      <div className="flex justify-end gap-2">
        <button
          type="button"
          onClick={async () => {
            await navigator.clipboard.writeText(password).catch(() => {});
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
          }}
          className="rounded border border-gray-300 px-2 py-1 text-xs hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700"
        >
          {copied ? <Check className="h-3.5 w-3.5" /> : '复制'}
        </button>
        <button
          type="button"
          onClick={() => {
            onUse(password);
            setOpen(false);
          }}
          className="rounded bg-primary-600 px-2 py-1 text-xs text-white hover:bg-primary-700"
        >
          使用
        </button>
      </div>
    </div>
  );
}
