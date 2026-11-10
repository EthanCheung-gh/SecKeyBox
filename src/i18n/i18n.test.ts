import { describe, expect, it } from 'vitest';
import { translate } from '@/i18n';
import { en } from '@/i18n/dictionaries/en';
import { zh } from '@/i18n/dictionaries/zh';

describe('i18n dictionaries', () => {
  it('defines the same keys in English and Chinese', () => {
    const enKeys = Object.keys(en).sort();
    const zhKeys = Object.keys(zh).sort();

    expect(enKeys).toEqual(zhKeys);
  });

  it('translates the trash and trash-retention settings keys in both languages', () => {
    const trashKeys = [
      'sidebar.trash',
      'trash.title',
      'trash.hint',
      'trash.hintNever',
      'trash.empty',
      'trash.restore',
      'trash.purge',
      'trash.emptyTrash',
      'trash.purgeConfirm',
      'trash.emptyConfirm',
      'trash.deletedAt',
      'trash.originalGroup',
      'trash.groupDeleted',
      'trash.deleteItemTitle',
      'trash.deleteItemBody',
      'trash.deleteGroupTitle',
      'trash.deleteGroupBody',
      'trash.deleteGroupItemsWarning',
      'trash.moveToTrash',
      'settings.trashRetention',
      'settings.trashRetentionHint',
      'settings.trashRetentionNever',
      'settings.days',
    ];

    for (const key of trashKeys) {
      expect(en[key], `en: ${key}`).toBeTruthy();
      expect(zh[key], `zh: ${key}`).toBeTruthy();
    }

    expect(translate('en', 'trash.title')).toBe('Trash');
    expect(translate('zh', 'trash.title')).toBe('回收站');
  });
});

describe('translate', () => {
  it('returns the active language string', () => {
    expect(translate('en', 'common.save')).toBe('Save');
    expect(translate('zh', 'common.save')).toBe('保存');
  });

  it('falls back to English when a key is missing from the active language', () => {
    const original = zh['common.save'];
    delete zh['common.save'];

    try {
      expect(translate('zh', 'common.save')).toBe('Save');
    } finally {
      zh['common.save'] = original;
    }
  });

  it('returns the key itself when it exists in no dictionary', () => {
    expect(translate('en', 'missing.key')).toBe('missing.key');
    expect(translate('zh', 'missing.key')).toBe('missing.key');
  });

  it('interpolates {name} variables', () => {
    expect(translate('en', 'setup.passwordTooShort', { count: 12 })).toBe(
      'Password must be at least 12 characters'
    );
    expect(translate('zh', 'setup.passwordTooShort', { count: 12 })).toBe('密码至少 12 位');
  });

  it('leaves unknown placeholders untouched', () => {
    expect(translate('en', 'setup.passwordTooShort', {})).toBe(
      'Password must be at least {count} characters'
    );
    expect(translate('en', 'setup.passwordTooShort')).toBe(
      'Password must be at least {count} characters'
    );
  });
});
