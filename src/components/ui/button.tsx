import { forwardRef } from 'react';
import { cn } from '@/lib/utils';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'default' | 'destructive' | 'outline' | 'ghost';
  size?: 'default' | 'sm' | 'lg';
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'default', size = 'default', ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          'inline-flex items-center justify-center rounded-md font-medium transition-colors',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500',
          'focus-visible:ring-offset-2 focus-visible:ring-offset-white',
          'dark:focus-visible:ring-offset-gray-800',
          'disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60',
          {
            'bg-primary-600 text-white hover:bg-primary-700': variant === 'default',
            'bg-red-600 text-white hover:bg-red-700': variant === 'destructive',
            'border border-gray-300 bg-white text-gray-700 hover:bg-gray-50': variant === 'outline',
            'dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 dark:hover:bg-gray-600':
              variant === 'outline',
            'text-gray-600 hover:bg-gray-100 hover:text-gray-900': variant === 'ghost',
            'dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-gray-100':
              variant === 'ghost',
            'h-9 px-4 text-sm': size === 'default',
            'h-8 px-3 text-xs': size === 'sm',
            'h-10 px-6 text-base': size === 'lg',
          },
          className
        )}
        {...props}
      />
    );
  }
);