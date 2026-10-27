import { useSettingsStore, type Language } from '@/stores/settings';
import { en } from './dictionaries/en';
import { zh } from './dictionaries/zh';

const dictionaries: Record<Language, Record<string, string>> = { en, zh };

export type TranslateVars = Record<string, string | number>;

/**
 * Look up `key` in the active language, falling back to English and then to
 * the key itself. `{name}` placeholders are replaced from `vars`.
 */
export function translate(language: Language, key: string, vars?: TranslateVars): string {
  const template = dictionaries[language]?.[key] ?? dictionaries.en[key] ?? key;
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (match, name: string) => {
    const value = vars[name];
    return value === undefined ? match : String(value);
  });
}

/** Reactive translator: re-renders the component when the language changes. */
export function useI18n() {
  const language = useSettingsStore((s) => s.language);
  return {
    language,
    t: (key: string, vars?: TranslateVars) => translate(language, key, vars),
  };
}

/** Non-reactive translation for callbacks that run outside render. */
export function t(key: string, vars?: TranslateVars): string {
  return translate(useSettingsStore.getState().language, key, vars);
}

export type { Language };

/** Built-in groups carry English names in the database; localize them. */
const BUILTIN_GROUP_KEYS: Record<string, string> = {
  'built-in-accounts': 'type.account',
  'built-in-api-keys': 'type.api_key',
  'built-in-env-vars': 'type.env_var',
  'built-in-databases': 'type.database',
  'built-in-servers': 'type.ssh',
  'built-in-cloud': 'type.cloud',
  'built-in-licenses': 'type.license',
  'built-in-smtp': 'type.smtp',
};

export function groupDisplayName(
  group: { id: string; name: string },
  t: (key: string, vars?: TranslateVars) => string
): string {
  const key = BUILTIN_GROUP_KEYS[group.id];
  return key ? t(key) : group.name;
}
