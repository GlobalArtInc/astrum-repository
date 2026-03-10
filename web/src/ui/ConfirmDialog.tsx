import { AlertDialog, Button, Flex, Text } from '@radix-ui/themes';
import { createContext, type ReactNode, useCallback, useContext, useRef, useState } from 'react';
import { useT } from '../i18n';
import { describeError } from './ErrorState';

export interface ConfirmOptions {
  title: string;
  description?: ReactNode;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}

export interface ConfirmDialogProps extends ConfirmOptions {
  open: boolean;
  onOpenChange(open: boolean): void;
  onConfirm(): void | Promise<void>;
}

export function ConfirmDialog({
  open,
  onOpenChange,
  onConfirm,
  title,
  description,
  confirmLabel,
  cancelLabel,
  danger = false,
}: ConfirmDialogProps) {
  const t = useT();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const change = (next: boolean) => {
    if (!busy) {
      setError(null);
      onOpenChange(next);
    }
  };
  const confirm = async () => {
    setBusy(true);
    setError(null);
    try {
      await onConfirm();
      onOpenChange(false);
    } catch (failure) {
      setError(failure);
    } finally {
      setBusy(false);
    }
  };
  return (
    <AlertDialog.Root open={open} onOpenChange={change}>
      <AlertDialog.Content maxWidth="460px">
        <AlertDialog.Title>{title}</AlertDialog.Title>
        <AlertDialog.Description size="2" color="gray">
          {description ?? ''}
        </AlertDialog.Description>
        {error !== null && (
          <Text as="p" size="2" color="red" mt="3" role="alert">
            {describeError(t, error)}
          </Text>
        )}
        <Flex gap="3" mt="4" justify="end" wrap="wrap">
          <AlertDialog.Cancel>
            <Button variant="soft" color="gray" disabled={busy}>
              {cancelLabel ?? t('ui.confirm.cancel')}
            </Button>
          </AlertDialog.Cancel>
          <Button color={danger ? 'red' : undefined} loading={busy} onClick={confirm}>
            {confirmLabel ?? t('ui.confirm.confirm')}
          </Button>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}

type Ask = (options: ConfirmOptions) => Promise<boolean>;

const ConfirmContext = createContext<Ask | null>(null);

export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [options, setOptions] = useState<ConfirmOptions | null>(null);
  const resolver = useRef<((answer: boolean) => void) | null>(null);
  const ask = useCallback<Ask>((next) => {
    resolver.current?.(false);
    setOptions(next);
    return new Promise<boolean>((resolve) => {
      resolver.current = resolve;
    });
  }, []);
  const settle = (answer: boolean) => {
    resolver.current?.(answer);
    resolver.current = null;
  };
  return (
    <ConfirmContext.Provider value={ask}>
      {children}
      <ConfirmDialog
        {...(options ?? { title: '' })}
        open={options !== null}
        onOpenChange={(open) => {
          if (!open) {
            settle(false);
            setOptions(null);
          }
        }}
        onConfirm={() => settle(true)}
      />
    </ConfirmContext.Provider>
  );
}

export function useConfirm(): Ask {
  const ask = useContext(ConfirmContext);
  if (!ask) {
    throw new Error('useConfirm needs a ConfirmProvider');
  }
  return ask;
}
