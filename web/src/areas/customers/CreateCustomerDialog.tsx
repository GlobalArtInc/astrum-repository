import { Button, Dialog, Flex } from '@radix-ui/themes';
import { api, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { FormError, TextAreaField, TextField, useForm, useToast } from '../../ui';
import { CUSTOMERS } from './api';
import {
  EMPTY_CUSTOMER,
  idProblem,
  NAME_MAX,
  type NewCustomerValues,
  REFERENCE_MAX,
  toNewCustomerRequest,
  validateNewCustomer,
} from './model';

export interface CreateCustomerDialogProps {
  open: boolean;
  onOpenChange(open: boolean): void;
}

export function CreateCustomerDialog({ open, onOpenChange }: CreateCustomerDialogProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const form = useForm<NewCustomerValues>({
    initial: EMPTY_CUSTOMER,
    validate: (values) => validateNewCustomer(t, values),
    onSubmit: async (values) => {
      await api.post(CUSTOMERS, toNewCustomerRequest(values));
      toast.success(t('customers.create.created', { id: values.id }));
      await invalidate(CUSTOMERS);
      onOpenChange(false);
    },
  });
  const liveIdError = form.values.id === '' ? undefined : idProblem(t, form.values.id);
  const change = (next: boolean) => {
    if (!form.submitting) {
      onOpenChange(next);
      form.reset(EMPTY_CUSTOMER);
    }
  };
  return (
    <Dialog.Root open={open} onOpenChange={change}>
      <Dialog.Content maxWidth="480px">
        <Dialog.Title>{t('customers.create.title')}</Dialog.Title>
        <Dialog.Description size="2" color="gray">
          {t('customers.create.intro')}
        </Dialog.Description>
        <form onSubmit={form.submit} noValidate>
          <Flex direction="column" gap="4" mt="4">
            <TextField
              {...form.field('id')}
              label={t('customers.create.id')}
              hint={t('customers.create.idHint')}
              error={form.errors.id ?? liveIdError}
              required
              maxLength={64}
              autoComplete="off"
            />
            <TextField
              {...form.field('name')}
              label={t('customers.create.name')}
              required
              maxLength={NAME_MAX}
              autoComplete="off"
            />
            <TextField
              {...form.field('reference')}
              label={t('customers.create.reference')}
              hint={t('customers.create.referenceHint')}
              maxLength={REFERENCE_MAX}
              autoComplete="off"
            />
            <TextAreaField {...form.field('note')} label={t('customers.create.note')} />
            <FormError message={form.formError} />
          </Flex>
          <Flex gap="3" mt="5" justify="end">
            <Dialog.Close>
              <Button type="button" variant="soft" color="gray" disabled={form.submitting}>
                {t('customers.create.cancel')}
              </Button>
            </Dialog.Close>
            <Button type="submit" loading={form.submitting}>
              {t('customers.create.submit')}
            </Button>
          </Flex>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
