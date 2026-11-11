import { Sun, Moon, Monitor } from 'lucide-react';
import { useSettingsStore } from '@/stores/settings';
import { useI18n } from '@/i18n';

export function ThemeToggle() {
  const { t } = useI18n();
  const theme = useSettingsStore((s) => s.theme);
  const setTheme = useSettingsStore((s) => s.setTheme);

  const cycleTheme = () => {
    const next = theme === 'light' ? 'dark' : theme === 'dark' ? 'system' : 'light';
    setTheme(next);
  };

  return (
    <button
      onClick={cycleTheme}
      className="rounded-md p-2 text-gray-500 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
      title={t(`theme.${theme}`)}
    >
      {theme === 'light' && <Sun className="h-5 w-5" />}
      {theme === 'dark' && <Moon className="h-5 w-5" />}
      {theme === 'system' && <Monitor className="h-5 w-5" />}
    </button>
  );
}