import { Button, Card, Flex } from '@radix-ui/themes';
import { api, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { FormError, SwitchField, TextAreaField, TextField, useForm, useToast } from '../../ui';
import { CUSTOMERS, type Customer, customerUrl } from './api';
import {
  type CustomerValues,
  fromCustomer,
  NAME_MAX,
  REFERENCE_MAX,
  toCustomerRequest,
  validateCustomer,
} from './model';

export interface CustomerFormProps {
  customer: Customer;
  canUpdate: boolean;
}

export function CustomerForm({ customer, canUpdate }: CustomerFormProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const form = useForm<CustomerValues>({
    initial: fromCustomer(customer),
    validate: (values) => validateCustomer(t, values),
    onSubmit: async (values) => {
      await api.put(customerUrl(customer.id), toCustomerRequest(values));
      toast.success(t('customers.detail.saved', { id: customer.id }));
      await invalidate(CUSTOMERS);
    },
  });
  return (
    <Card>
      <form onSubmit={form.submit} noValidate>
        <Flex direction="column" gap="4">
          <TextField
            {...form.field('name')}
            label={t('customers.detail.name')}
            required
            disabled={!canUpdate}
            maxLength={NAME_MAX}
            autoComplete="off"
          />
          <TextField
            {...form.field('reference')}
            label={t('customers.detail.reference')}
            disabled={!canUpdate}
            maxLength={REFERENCE_MAX}
            autoComplete="off"
          />
          <TextAreaField {...form.field('note')} label={t('customers.detail.note')} disabled={!canUpdate} />
          <SwitchField
            {...form.field('disabled')}
            label={t('customers.detail.disabled')}
            description={t('customers.detail.disabledHint')}
            disabled={!canUpdate}
          />
          <FormError message={form.formError} />
          {canUpdate && (
            <Flex justify="end">
              <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
                {t('customers.detail.save')}
              </Button>
            </Flex>
          )}
        </Flex>
      </form>
    </Card>
  );
}
