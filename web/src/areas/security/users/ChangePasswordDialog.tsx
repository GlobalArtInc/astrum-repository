import { Button, Dialog, Flex } from '@radix-ui/themes';
import { api, path } from '../../../api';
import { useT } from '../../../i18n';
import { FormError, PasswordField, useForm, useToast } from '../../../ui';
import { USERS } from '../api';
import { explainRefusal } from '../shared/messages';
import { passwordProblems } from './model';

interface Values {
  password: string;
  confirm: string;
}

function PasswordForm({ userId, onDone }: { userId: string; onDone(): void }) {
  const t = useT();
  const toast = useToast();
  const form = useForm<Values>({
    initial: { password: '', confirm: '' },
    validate: (values) => passwordProblems(t, values.password, values.confirm),
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        await api.put(`${USERS}${path`/${userId}/change-password`}`, values.password, {
          headers: { 'Content-Type': 'text/plain; charset=utf-8' },
        });
        toast.success(t('security.password.changed', { id: userId }));
        onDone();
      }),
  });
  return (
    <form onSubmit={form.submit} noValidate>
      <Flex direction="column" gap="4">
        <PasswordField
          label={t('security.password.new')}
          {...form.field('password')}
          required
          autoFocus
          hint={t('security.password.hint')}
        />
        <PasswordField label={t('security.password.confirm')} {...form.field('confirm')} required />
        <FormError message={form.formError} />
        <Flex gap="3" justify="end" wrap="wrap">
          <Dialog.Close>
            <Button type="button" variant="soft" color="gray">
              {t('security.form.cancel')}
            </Button>
          </Dialog.Close>
          <Button type="submit" loading={form.submitting}>
            {t('security.password.submit')}
          </Button>
        </Flex>
      </Flex>
    </form>
  );
}

export function ChangePasswordDialog({
  userId,
  open,
  onOpenChange,
}: {
  userId: string;
  open: boolean;
  onOpenChange(open: boolean): void;
}) {
  const t = useT();
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="460px">
        <Dialog.Title>{t('security.password.title', { id: userId })}</Dialog.Title>
        <Dialog.Description size="2" color="gray" mb="4">
          {t('security.password.description')}
        </Dialog.Description>
        {open && <PasswordForm userId={userId} onDone={() => onOpenChange(false)} />}
      </Dialog.Content>
    </Dialog.Root>
  );
}
