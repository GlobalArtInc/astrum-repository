import { AlertDialog, Button, Flex, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { useT } from '../../../i18n';
import { describeError } from '../../../ui';
import { TextInput } from '../../repositories/components/fields';
import { NAME_PATTERN } from '../model';

export interface ConvertDialogProps {
  name: string;
  open: boolean;
  onOpenChange(open: boolean): void;
  onConvert(member: string): Promise<void>;
}

export function ConvertDialog({ name, open, onOpenChange, onConvert }: ConvertDialogProps) {
  const t = useT();
  const [member, setMember] = useState(`${name}-original`);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<unknown>(null);
  const trimmed = member.trim();
  const invalid =
    trimmed === ''
      ? t('blobstores.form.errors.nameRequired')
      : NAME_PATTERN.test(trimmed)
        ? undefined
        : t('blobstores.form.errors.namePattern');
  const change = (next: boolean) => {
    if (!busy) {
      setFailure(null);
      onOpenChange(next);
    }
  };
  const convert = async () => {
    if (invalid) {
      return;
    }
    setBusy(true);
    setFailure(null);
    try {
      await onConvert(trimmed);
      onOpenChange(false);
    } catch (error) {
      setFailure(error);
    } finally {
      setBusy(false);
    }
  };
  return (
    <AlertDialog.Root open={open} onOpenChange={change}>
      <AlertDialog.Content maxWidth="520px">
        <AlertDialog.Title>{t('blobstores.convert.title', { name })}</AlertDialog.Title>
        <AlertDialog.Description size="2" color="gray">
          {t('blobstores.convert.text', { name })}
        </AlertDialog.Description>
        <Flex direction="column" mt="4">
          <TextInput
            label={t('blobstores.convert.member')}
            value={member}
            onChange={setMember}
            error={invalid}
            disabled={busy}
            required
          />
        </Flex>
        {failure !== null && (
          <Text as="p" size="2" color="red" mt="3" role="alert">
            {describeError(t, failure)}
          </Text>
        )}
        <Flex gap="3" mt="4" justify="end" wrap="wrap">
          <AlertDialog.Cancel>
            <Button variant="soft" color="gray" disabled={busy}>
              {t('ui.confirm.cancel')}
            </Button>
          </AlertDialog.Cancel>
          <Button loading={busy} disabled={invalid !== undefined} onClick={() => void convert()}>
            {t('blobstores.convert.confirm')}
          </Button>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}
