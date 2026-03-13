export type StrengthLevel = 'weak' | 'fair' | 'good' | 'strong';

export interface StrengthResult {
  score: number;
  level: StrengthLevel;
  color: string;
  label: string;
}

export function calculatePasswordStrength(password: string): StrengthResult {
  if (!password) {
    return { score: 0, level: 'weak', color: 'bg-gray-300', label: '' };
  }

  let score = 0;

  score += Math.min(password.length * 4, 32);

  if (/[a-z]/.test(password)) score += 5;
  if (/[A-Z]/.test(password)) score += 5;
  if (/[0-9]/.test(password)) score += 5;
  if (/[^a-zA-Z0-9]/.test(password)) score += 10;

  if (/(.)\1{2,}/.test(password)) score -= 10;
  if (/^[a-z]+$/.test(password)) score -= 10;
  if (/^[A-Z]+$/.test(password)) score -= 10;
  if (/^[0-9]+$/.test(password)) score -= 15;

  if (/password/i.test(password)) score -= 15;
  if (/123456|qwerty|abc123/i.test(password)) score -= 20;

  score = Math.max(0, Math.min(100, score));

  if (score < 40) {
    return { score, level: 'weak', color: 'bg-red-500', label: 'Weak' };
  } else if (score < 60) {
    return { score, level: 'fair', color: 'bg-orange-500', label: 'Fair' };
  } else if (score < 80) {
    return { score, level: 'good', color: 'bg-yellow-500', label: 'Good' };
  } else {
    return { score, level: 'strong', color: 'bg-green-500', label: 'Strong' };
  }
}