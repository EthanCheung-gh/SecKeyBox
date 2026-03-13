import { calculatePasswordStrength } from '@/lib/password-strength';

interface PasswordStrengthProps {
  password: string;
}

export function PasswordStrength({ password }: PasswordStrengthProps) {
  const result = calculatePasswordStrength(password);

  if (!password) return null;

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between text-xs">
        <span className="text-gray-500 dark:text-gray-400">Password strength</span>
        <span className={`font-medium ${
          result.level === 'weak' ? 'text-red-500' :
          result.level === 'fair' ? 'text-orange-500' :
          result.level === 'good' ? 'text-yellow-600' :
          'text-green-500'
        }`}>
          {result.label}
        </span>
      </div>
      <div className="h-1.5 w-full rounded-full bg-gray-200 dark:bg-gray-700">
        <div
          className={`h-full rounded-full transition-all duration-300 ${result.color}`}
          style={{ width: `${result.score}%` }}
        />
      </div>
    </div>
  );
}