

import React, { createContext, useContext, useState, useCallback, useRef } from 'react';
import { createPortal } from 'react-dom';
import * as ToastPrimitive from '@radix-ui/react-toast';
import { CheckCircle2, XCircle, AlertCircle, Info, X } from 'lucide-react';
import './Toast.scss';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastMessage {
  id: string;
  type: ToastType;
  title: string;
  description?: string;
  duration?: number;
}

interface ToastContextValue {
  showToast: (type: ToastType, title: string, description?: string, duration?: number) => void;
  success: (title: string, description?: string) => void;
  error: (title: string, description?: string) => void;
  warning: (title: string, description?: string) => void;
  info: (title: string, description?: string) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

const TOAST_ICONS: Record<ToastType, React.ReactNode> = {
  success: <CheckCircle2 size={20} />,
  error: <XCircle size={20} />,
  warning: <AlertCircle size={20} />,
  info: <Info size={20} />,
};

const DEFAULT_DURATIONS: Record<ToastType, number> = {
  success: 4000,
  error: 6000,
  warning: 5000,
  info: 4000,
};

/**
 * Toast component that renders a single toast message
 */
function Toast({ toast, onOpenChange }: { toast: ToastMessage; onOpenChange: (open: boolean) => void }) {
  return (
    <ToastPrimitive.Root
      className={`toast toast--${toast.type}`}
      open={true}
      onOpenChange={onOpenChange}
      duration={toast.duration || DEFAULT_DURATIONS[toast.type]}
      data-testid={`toast-${toast.type}`}
    >
      <div className="toast__icon">{TOAST_ICONS[toast.type]}</div>
      <div className="toast__content">
        <ToastPrimitive.Title className="toast__title">{toast.title}</ToastPrimitive.Title>
        {toast.description && (
          <ToastPrimitive.Description className="toast__description">
            {toast.description}
          </ToastPrimitive.Description>
        )}
      </div>
      <ToastPrimitive.Close className="toast__close" aria-label="Close">
        <X size={16} />
      </ToastPrimitive.Close>
    </ToastPrimitive.Root>
  );
}

/**
 * ToastProvider - Wrap your app or page with this to enable toast notifications
 *
 * Usage:
 * ```tsx
 * <ToastProvider>
 *   <YourPage />
 * </ToastProvider>
 * ```
 *
 * Then in your component:
 * ```tsx
 * const { success, error } = useToast();
 * success('Saved successfully!');
 * error('Failed to save', 'Please check your connection');
 * ```
 */
export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);
  const toastIdRef = useRef(0);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const showToast = useCallback(
    (type: ToastType, title: string, description?: string, duration?: number) => {
      const id = `toast-${++toastIdRef.current}`;
      const newToast: ToastMessage = { id, type, title, description, duration };
      setToasts((prev) => [...prev, newToast]);
    },
    []
  );

  const success = useCallback(
    (title: string, description?: string) => showToast('success', title, description),
    [showToast]
  );

  const error = useCallback(
    (title: string, description?: string) => showToast('error', title, description),
    [showToast]
  );

  const warning = useCallback(
    (title: string, description?: string) => showToast('warning', title, description),
    [showToast]
  );

  const info = useCallback(
    (title: string, description?: string) => showToast('info', title, description),
    [showToast]
  );

  const contextValue: ToastContextValue = {
    showToast,
    success,
    error,
    warning,
    info,
  };

  return (
    <ToastContext.Provider value={contextValue}>
      <ToastPrimitive.Provider swipeDirection="right">
        {children}
        {toasts.map((toast) => (
          <Toast
            key={toast.id}
            toast={toast}
            onOpenChange={(open) => {
              if (!open) removeToast(toast.id);
            }}
          />
        ))}
        {typeof document !== 'undefined' && createPortal(
          <ToastPrimitive.Viewport className="toast-viewport" />,
          document.body
        )}
      </ToastPrimitive.Provider>
    </ToastContext.Provider>
  );
}

/**
 * Hook to access toast functions
 *
 * @returns Toast context with success, error, warning, info methods
 * @throws Error if used outside ToastProvider
 *
 * @example
 * ```tsx
 * const { success, error } = useToast();
 *
 * const handleSave = async () => {
 *   try {
 *     await save();
 *     success('Settings saved successfully');
 *   } catch (err) {
 *     error('Failed to save settings', err.message);
 *   }
 * };
 * ```
 */
// No-op fallback when ToastProvider is not in the tree
const noopToast: ToastContextValue = {
  showToast: () => {},
  success: () => {},
  error: () => {},
  warning: () => {},
  info: () => {},
};

export function useToast(): ToastContextValue {
  const context = useContext(ToastContext);
  if (!context) {
    return noopToast;
  }
  return context;
}

export default ToastProvider;
