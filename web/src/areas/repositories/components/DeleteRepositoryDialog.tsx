import { AlertDialog, Button, Flex, Text, TextField } from '@radix-ui/themes';
import { useId, useState } from 'react';
import type { RepositoryType } from '../../../api';
import { useT } from '../../../i18n';
import { describeError } from '../../../ui';

const EXPLANATIONS: Record<RepositoryType, string> = {
  hosted: 'repositories.delete.hosted',
  proxy: 'repositories.delete.proxy',
  group: 'repositories.delete.group',
};

export interface DeleteRepositoryDialogProps {
  open: boolean;
  onOpenChange(open: boolean): void;
  name: string;
  type: RepositoryType;
  groups: readonly string[];
  onDelete(): Promise<void>;
}

export function DeleteRepositoryDialog({
  open,
  onOpenChange,
  name,
  type,
  groups,
  onDelete,
}: DeleteRepositoryDialogProps) {
  const t = useT();
  const inputId = useId();
  const [typed, setTyped] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const change = (next: boolean) => {
    if (busy) {
      return;
    }
    setTyped('');
    setError(null);
    onOpenChange(next);
  };
  const confirm = async () => {
    setBusy(true);
    setError(null);
    try {
      await onDelete();
      setTyped('');
      onOpenChange(false);
    } catch (failure) {
      setError(failure);
    } finally {
      setBusy(false);
    }
  };
  const matches = typed === name;
  return (
    <AlertDialog.Root open={open} onOpenChange={change}>
      <AlertDialog.Content maxWidth="500px">
        <AlertDialog.Title>{t('repositories.delete.title', { name })}</AlertDialog.Title>
        <AlertDialog.Description size="2">
          {t(EXPLANATIONS[type])}{' '}
          {groups.length > 0 && t('repositories.delete.groups', { groups: groups.join(', ') })}{' '}
          {t('repositories.delete.undone')}
        </AlertDialog.Description>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (matches && !busy) {
              void confirm();
            }
          }}
        >
          <Flex direction="column" gap="1" mt="4">
            <Text as="label" htmlFor={inputId} size="2" weight="medium">
              {t('repositories.delete.confirmLabel', { name })}
            </Text>
            <TextField.Root
              id={inputId}
              value={typed}
              autoComplete="off"
              spellCheck={false}
              onChange={(event) => setTyped(event.target.value)}
            />
          </Flex>
          {error !== null && (
            <Text as="p" size="2" color="red" mt="3" role="alert">
              {describeError(t, error)}
            </Text>
          )}
          <Flex gap="3" mt="4" justify="end" wrap="wrap">
            <AlertDialog.Cancel>
              <Button type="button" variant="soft" color="gray" disabled={busy}>
                {t('repositories.delete.cancel')}
              </Button>
            </AlertDialog.Cancel>
            <Button type="submit" color="red" disabled={!matches} loading={busy}>
              {t('repositories.delete.confirm')}
            </Button>
          </Flex>
        </form>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}
