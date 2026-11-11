import { Flex, Spinner } from '@radix-ui/themes';
import { useParams } from 'react-router';
import { useApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ErrorState, Page } from '../../ui';
import { CUSTOMERS_PAGE, type Customer, customerUrl } from './api';
import { CustomerForm } from './CustomerForm';
import { CustomerTokens } from './CustomerTokens';
import { CustomerUsage } from './CustomerUsage';
import { DeleteCustomer } from './DeleteCustomer';
import { CREATE, DELETE, UPDATE } from './model';

export default function CustomerPage() {
  const t = useT();
  const { id = '' } = useParams();
  const canCreate = useCan(CREATE);
  const canUpdate = useCan(UPDATE);
  const canDelete = useCan(DELETE);
  const customer = useApi<Customer>(customerUrl(id));
  return (
    <Page
      gutter="small"
      title={customer.data?.name ?? id}
      description={t('customers.detail.intro')}
      breadcrumbs={[
        { label: t('customers.crumbs.settings'), to: '/settings' },
        { label: t('customers.list.title'), to: CUSTOMERS_PAGE },
        { label: id },
      ]}
    >
      {customer.error ? (
        <ErrorState error={customer.error} onRetry={() => void customer.refetch()} />
      ) : customer.data === undefined ? (
        <Flex justify="center" py="6">
          <Spinner />
        </Flex>
      ) : (
        <Flex direction="column" gap="5">
          <CustomerForm customer={customer.data} canUpdate={canUpdate} />
          <CustomerTokens customerId={customer.data.id} canIssue={canCreate} canRevoke={canDelete} />
          <CustomerUsage customerId={customer.data.id} />
          {canDelete && <DeleteCustomer customer={customer.data} />}
        </Flex>
      )}
    </Page>
  );
}
