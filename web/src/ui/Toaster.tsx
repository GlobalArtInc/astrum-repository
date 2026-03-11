import { IconButton } from '@radix-ui/themes';
import { CircleAlert, CircleCheck, Info, X } from 'lucide-react';
import { Toast } from 'radix-ui';
import { createContext, type ReactNode, useCallback, useContext, useMemo, useState } from 'react';
import { useT } from '../i18n';
import styles from './Toaster.module.css';

export type ToastTone = 'success' | 'error' | 'info';

export interface ToastMessage {
  title: string;
  description?: string;
  tone?: ToastTone;
}

interface ShownToast extends Required<Pick<ToastMessage, 'title' | 'tone'>> {
  id: number;
  description?: string;
}

export interface Toasts {
  show(message: ToastMessage): void;
  success(title: string, description?: string): void;
  error(title: string, description?: string): void;
}

const ToastContext = createContext<Toasts | null>(null);
let nextId = 1;

const ICONS = { success: CircleCheck, error: CircleAlert, info: Info } as const;
const COLORS = { success: 'var(--green-9)', error: 'var(--red-9)', info: 'var(--accent-9)' } as const;

export function Toaster({ children }: { children: ReactNode }) {
  const t = useT();
  const [toasts, setToasts] = useState<ShownToast[]>([]);
  const show = useCallback((message: ToastMessage) => {
    const toast: ShownToast = {
      id: nextId++,
      title: message.title,
      description: message.description,
      tone: message.tone ?? 'info',
    };
    setToasts((current) => [...current.slice(-4), toast]);
  }, []);
  const value = useMemo<Toasts>(
    () => ({
      show,
      success: (title, description) => show({ title, description, tone: 'success' }),
      error: (title, description) => show({ title, description, tone: 'error' }),
    }),
    [show],
  );
  const dismiss = (id: number) => setToasts((current) => current.filter((toast) => toast.id !== id));
  return (
    <ToastContext.Provider value={value}>
      <Toast.Provider swipeDirection="right" duration={6000} label={t('ui.toast.region')}>
        {children}
        {toasts.map((toast) => {
          const Icon = ICONS[toast.tone];
          return (
            <Toast.Root
              key={toast.id}
              className={styles.toast}
              type={toast.tone === 'error' ? 'foreground' : 'background'}
              onOpenChange={(open) => {
                if (!open) {
                  dismiss(toast.id);
                }
              }}
            >
              <Icon size={18} className={styles.icon} color={COLORS[toast.tone]} aria-hidden="true" />
              <div>
                <Toast.Title className={styles.title}>{toast.title}</Toast.Title>
                {toast.description && (
                  <Toast.Description className={styles.description}>{toast.description}</Toast.Description>
                )}
              </div>
              <Toast.Close asChild>
                <IconButton size="1" variant="ghost" color="gray" aria-label={t('ui.toast.close')}>
                  <X size={14} aria-hidden="true" />
                </IconButton>
              </Toast.Close>
            </Toast.Root>
          );
        })}
        <Toast.Viewport className={styles.viewport} />
      </Toast.Provider>
    </ToastContext.Provider>
  );
}

const SILENT: Toasts = { show: () => {}, success: () => {}, error: () => {} };

export function useToast(): Toasts {
  return useContext(ToastContext) ?? SILENT;
}
