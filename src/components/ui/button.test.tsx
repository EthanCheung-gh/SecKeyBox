import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import { Button } from '@/components/ui/button';

describe('Button', () => {
  it('renders an accessible button and forwards clicks', () => {
    const handler = vi.fn();

    render(<Button onClick={handler}>Save</Button>);

    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toBeInTheDocument();

    fireEvent.click(button);
    expect(handler).toHaveBeenCalledTimes(1);
  });
});
