import { beforeEach, describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { IntegrityReportView } from '@/components/security/IntegrityReportView';
import { useSettingsStore } from '@/stores/settings';
import type { IntegrityReport } from '@/types';

const healthyReport: IntegrityReport = {
  checked_fields: 12,
  failures: [],
  sqlite_integrity: 'ok',
  foreign_key_violations: [],
  checked_attachments: false,
  checked_at: 1_700_000_000,
  duration_ms: 42,
};

const failingReport: IntegrityReport = {
  ...healthyReport,
  failures: [
    {
      item_id: 'item-1',
      item_title: 'GitHub',
      item_type: 'account',
      scope: 'account',
      field: 'password',
      reason: 'decryption failed: aead::Error',
    },
  ],
};

describe('IntegrityReportView', () => {
  beforeEach(() => {
    useSettingsStore.setState({ language: 'en' });
  });

  it('reports a healthy vault with the checked field count', () => {
    render(<IntegrityReportView report={healthyReport} />);

    expect(
      screen.getByText('All 12 encrypted fields decrypted successfully.')
    ).toBeInTheDocument();
    expect(screen.getByText('Checked fields: 12')).toBeInTheDocument();
    expect(screen.getByText('SQLite structure: ok')).toBeInTheDocument();
    expect(screen.getByText('Foreign key violations: 0')).toBeInTheDocument();
    // The restore hint only makes sense when something is wrong.
    expect(screen.queryByText(/restore the latest local backup/)).not.toBeInTheDocument();
  });

  it('lists each undecryptable field without exposing secrets', () => {
    render(<IntegrityReportView report={failingReport} />);

    expect(screen.getByText('1 field(s) could not be decrypted.')).toBeInTheDocument();
    expect(screen.getByText('GitHub')).toBeInTheDocument();
    expect(screen.getByText('· Account')).toBeInTheDocument();
    expect(screen.getByText('account · password')).toBeInTheDocument();
    expect(screen.getByText('decryption failed: aead::Error')).toBeInTheDocument();
    expect(screen.getByText(/restore the latest local backup/)).toBeInTheDocument();
  });
});
