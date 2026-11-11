interface DialogProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
}

export function Dialog({ open, onClose, title, children }: DialogProps) {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black/50" onClick={onClose} />
      {/* Content taller than the window must stay reachable: cap the dialog
          height and let the body scroll instead of clipping the actions. */}
      <div className="relative z-50 w-full max-w-md max-h-[90vh] overflow-y-auto rounded-lg bg-white p-6 shadow-lg dark:bg-gray-800 dark:text-gray-100">
        {title && <h2 className="mb-4 text-lg font-semibold dark:text-gray-100">{title}</h2>}
        {children}
      </div>
    </div>
  );
}