import { AlertDialog, Button, Callout, Flex, Text, TextField } from '@radix-ui/themes';
import { useId, useState } from 'react';
import { useT } from '../../../i18n';
import { describeError } from '../../../ui';

export function DeleteTaskDialog({
  open,
  onOpenChange,
  name,
  onDelete,
}: {
  open: boolean;
  onOpenChange(open: boolean): void;
  name: string;
  onDelete(): Promise<void>;
}) {
  const t = useT();
  const inputId = useId();
  const [typed, setTyped] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const word = t('tasks.delete.word');
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
  return (
    <AlertDialog.Root open={open} onOpenChange={change}>
      <AlertDialog.Content maxWidth="460px">
        <AlertDialog.Title>{t('tasks.delete.title')}</AlertDialog.Title>
        <AlertDialog.Description size="2">
          {t.rich('tasks.delete.text', { name: <strong key="name">{name}</strong> })}
        </AlertDialog.Description>
        <Callout.Root color="red" variant="surface" mt="4">
          <Callout.Text>
            <Text as="span" weight="medium">
              {word}
            </Text>
            <br />
            <Text as="span" size="1">
              {t('tasks.delete.typeThis')}
            </Text>
          </Callout.Text>
        </Callout.Root>
        <Flex direction="column" gap="1" mt="4">
          <Text as="label" htmlFor={inputId} size="2" weight="bold">
            {t('tasks.delete.acknowledgement')}
          </Text>
          <TextField.Root
            id={inputId}
            value={typed}
            placeholder={t('tasks.delete.placeholder', { word })}
            onChange={(event) => setTyped(event.target.value)}
          />
        </Flex>
        {error !== null && (
          <Text as="p" size="2" color="red" mt="3" role="alert">
            {describeError(t, error)}
          </Text>
        )}
        <Flex gap="3" mt="4" justify="end">
          <AlertDialog.Cancel>
            <Button variant="soft" color="gray" disabled={busy}>
              {t('tasks.actions.cancel')}
            </Button>
          </AlertDialog.Cancel>
          <Button color="red" loading={busy} disabled={typed.trim() !== word} onClick={() => void confirm()}>
            {t('tasks.actions.delete')}
          </Button>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}
