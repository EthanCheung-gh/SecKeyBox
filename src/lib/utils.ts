import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString();
}

export function truncate(str: string, length: number): string {
  if (str.length <= length) return str;
  return str.slice(0, length) + '...';
}

export function maskKeyValue(keyValue: string): string {
  if (keyValue.length <= 4) return '••••';
  const lastFour = keyValue.slice(-4);
  return `••••${lastFour}`;
}

export function formatRotationDate(timestamp: number | undefined): string {
  if (!timestamp) return 'Never rotated';
  const days = Math.floor((Date.now() / 1000 - timestamp) / 86400);
  if (days === 0) return 'Rotated today';
  if (days === 1) return 'Rotated yesterday';
  return `Rotated ${days} days ago`;
}

export function formatExpiryDate(timestamp: number | undefined): string {
  if (!timestamp) return 'Never expires';
  const days = Math.floor((timestamp - Date.now() / 1000) / 86400);
  const date = new Date(timestamp * 1000).toLocaleDateString();
  if (days < 0) return `${date} (expired ${-days} days ago)`;
  if (days === 0) return `${date} (expires today)`;
  return `${date} (expires in ${days} days)`;
}