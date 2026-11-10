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

    expect(result.score).toBe(50);
    expect(result.level).toBe('fair');
    expect(result.label).toBe('Fair');
  });

  it('keeps scores below 40 at the weak level', () => {
    const result = calculatePasswordStrength('abcdefghijk');

    expect(result.score).toBe(39);
    expect(result.level).toBe('weak');
  });

  it('reaches good with a 10-character password using every character class', () => {
    // 40 length points + 25 variety points, no length bonus yet.
    const result = calculatePasswordStrength('Abcdefg1!x');

    expect(result.score).toBe(65);
    expect(result.level).toBe('good');
    expect(result.label).toBe('Good');
  });

  it('reaches strong with a 16-character mixed password', () => {
    // 48 length points + 25 variety points + 12 length-bonus points.
    const result = calculatePasswordStrength('Abcdefghij1!Klmn');

    expect(result.score).toBe(85);
    expect(result.level).toBe('strong');
    expect(result.label).toBe('Strong');
  });

  it('caps at 88 points for a 20-character password using every character class', () => {
    // 48 length points + 25 variety points + 15 length-bonus points.
    const result = calculatePasswordStrength('Abcdefghij1!KlmnoPqr');

    expect(result.score).toBe(88);
    expect(result.level).toBe('strong');
  });
});
