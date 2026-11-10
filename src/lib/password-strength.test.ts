import { describe, expect, it } from 'vitest';
import { calculatePasswordStrength } from '@/lib/password-strength';

describe('calculatePasswordStrength', () => {
  it('scores an empty password as weak', () => {
    expect(calculatePasswordStrength('')).toEqual({
      score: 0,
      level: 'weak',
      color: 'bg-gray-300',
      label: '',
    });
  });

  it('scores a short lowercase word as weak', () => {
    const result = calculatePasswordStrength('abc');

    expect(result.score).toBe(7);
    expect(result.level).toBe('weak');
    expect(result.label).toBe('Weak');
  });

  it('scores a common numeric password as weak', () => {
    const result = calculatePasswordStrength('123456');

    expect(result.score).toBe(0);
    expect(result.level).toBe('weak');
  });

  it('penalizes repeated characters', () => {
    const result = calculatePasswordStrength('aaaaaaaa');

    expect(result.score).toBe(17);
    expect(result.level).toBe('weak');
  });

  it('promotes mixed-case passwords to fair at the 40-point threshold', () => {
    const result = calculatePasswordStrength('Abcdefghij');

    expect(result.score).toBe(42);
    expect(result.level).toBe('fair');
    expect(result.label).toBe('Fair');
  });

  it('keeps a long lowercase-only password at the weak level', () => {
    const result = calculatePasswordStrength('abcdefghijk');

    expect(result.score).toBe(27);
    expect(result.level).toBe('weak');
  });

  it('caps the current scoring model at 57 points', () => {
    // 20 chars (32 length points) + lower/upper/digit/symbol (25 points).
    // The cap is intentional here: it documents the pre-fix scoring ceiling.
    const result = calculatePasswordStrength('Abcdefghij1!KlmnoPqr');

    expect(result.score).toBe(57);
    expect(result.level).toBe('fair');
  });
});
