import { AlertDialog, Button, Flex, Text } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router';
import { api, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { describeError, TextField, useToast } from '../../ui';
import { CUSTOMERS, CUSTOMERS_PAGE, type Customer, customerUrl } from './api';

export interface DeleteCustomerProps {
  customer: Customer;
}

export function DeleteCustomer({ customer }: DeleteCustomerProps) {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const [open, setOpen] = useState(false);
  const [typed, setTyped] = useState('');
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const matches = typed === customer.id;
  const change = (next: boolean) => {
    if (!busy) {
      setOpen(next);
      setTyped('');
      setFailure(null);
    }
  };
  const remove = async () => {
    setBusy(true);
    setFailure(null);
    try {
      await api.delete(customerUrl(customer.id));
      toast.success(t('customers.delete.done', { id: customer.id }));
      await invalidate(CUSTOMERS);
      navigate(CUSTOMERS_PAGE);
    } catch (error) {
      setFailure(describeError(t, error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Flex justify="end">
      <Button color="red" variant="outline" onClick={() => setOpen(true)}>
        <Trash2 size={16} aria-hidden="true" />
        {t('customers.detail.delete')}
      </Button>
      <AlertDialog.Root open={open} onOpenChange={change}>
        <AlertDialog.Content maxWidth="460px">
          <AlertDialog.Title>{t('customers.delete.title')}</AlertDialog.Title>
          <AlertDialog.Description size="2" color="gray">
            {t('customers.delete.text', { name: customer.name })}
          </AlertDialog.Description>
          <Flex direction="column" mt="4">
            <TextField
              name="confirmId"
              label={t('customers.delete.label')}
              hint={t('customers.delete.hint', { id: customer.id })}
              value={typed}
              onChange={setTyped}
              autoComplete="off"
            />
          </Flex>
          {failure !== null && (
            <Text as="p" size="2" color="red" mt="3" role="alert">
              {failure}
            </Text>
          )}
          <Flex gap="3" mt="4" justify="end" wrap="wrap">
            <AlertDialog.Cancel>
              <Button variant="soft" color="gray" disabled={busy}>
                {t('customers.delete.cancel')}
              </Button>
            </AlertDialog.Cancel>
            <Button color="red" disabled={!matches} loading={busy} onClick={remove}>
              {t('customers.delete.confirm')}
            </Button>
          </Flex>
        </AlertDialog.Content>
      </AlertDialog.Root>
    </Flex>
  );
}
