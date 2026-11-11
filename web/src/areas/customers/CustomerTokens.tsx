import { Badge, Button, Card, Flex, Heading, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { api, path, useApi, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { type Column, ConfirmDialog, DataTable, RelativeTime, type RowAction, useToast } from '../../ui';
import { CUSTOMERS, type CustomerToken, customerUrl, type RevokeResult, tokensUrl } from './api';
import styles from './Customers.module.css';
import { Expiry } from './Expiry';

const MAX_REPOSITORIES = 3;

export interface CustomerTokensProps {
  customerId: string;
  canRevoke: boolean;
}

export interface RepositoriesProps {
  names: string[];
}

function Repositories({ names }: RepositoriesProps) {
  const t = useT();
  const shown = names.slice(0, MAX_REPOSITORIES);
  const more = names.length - shown.length;
  return (
    <Flex wrap="wrap" gap="1" align="center">
      {shown.map((name) => (
        <Badge key={name} color="gray" variant="outline">
          {name}
        </Badge>
      ))}
      {more > 0 && (
        <Badge color="gray" variant="soft">
          {t('customers.tokens.more', { count: more })}
        </Badge>
      )}
    </Flex>
  );
}

export function CustomerTokens({ customerId, canRevoke }: CustomerTokensProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const tokens = useApi<CustomerToken[]>(tokensUrl(customerId));
  const [revoking, setRevoking] = useState<CustomerToken | null>(null);
  const [revokingAll, setRevokingAll] = useState(false);
  const rows = tokens.data ?? [];
  const columns: Column<CustomerToken>[] = [
    {
      id: 'name',
      header: t('customers.tokens.name'),
      primary: true,
      sortValue: (row) => row.name,
      cell: (row) => row.name,
    },
    {
      id: 'hint',
      header: t('customers.tokens.hint'),
      cell: (row) => <code className={styles.code}>{row.hint}</code>,
    },
    {
      id: 'owner',
      header: t('customers.tokens.owner'),
      sortValue: (row) => row.owner,
      cell: (row) => row.owner,
    },
    {
      id: 'repositories',
      header: t('customers.tokens.repositories'),
      cell: (row) => <Repositories names={row.repositories} />,
    },
    {
      id: 'expires',
      header: t('customers.tokens.expires'),
      sortValue: (row) => row.expiresAt ?? '',
      cell: (row) => (row.expiresAt === null ? t('customers.tokens.never') : <Expiry at={row.expiresAt} />),
    },
    {
      id: 'lastUsed',
      header: t('customers.tokens.lastUsed'),
      sortValue: (row) => row.lastUsedAt ?? '',
      cell: (row) =>
        row.lastUsedAt === null ? (
          t('customers.tokens.neverUsed')
        ) : (
          <>
            <RelativeTime value={row.lastUsedAt} />
            {row.lastUsedIp && <span className={styles.address}> {row.lastUsedIp}</span>}
          </>
        ),
    },
    {
      id: 'createdBy',
      header: t('customers.tokens.createdBy'),
      cell: (row) => row.createdBy,
    },
  ];
  const rowActions = (row: CustomerToken): RowAction<CustomerToken>[] => [
    {
      id: 'revoke',
      label: t('customers.tokens.revoke'),
      danger: true,
      onSelect: () => setRevoking(row),
    },
  ];
  const revokeOne = async () => {
    if (revoking === null) {
      return;
    }
    await api.delete(`${tokensUrl(customerId)}${path`/${revoking.id}`}`);
    toast.success(t('customers.tokens.revoked', { name: revoking.name }));
    await invalidate(CUSTOMERS);
  };
  const revokeAll = async () => {
    const result = await api.post<RevokeResult>(`${customerUrl(customerId)}/revoke`);
    toast.success(t('customers.tokens.revokedAll', { count: result.revoked }));
    await invalidate(CUSTOMERS);
  };
  return (
    <Card>
      <Flex justify="between" align="center" gap="3" wrap="wrap" mb="3">
        <Heading as="h2" size="3" weight="medium">
          {t('customers.tokens.title')}
        </Heading>
        {canRevoke && (
          <Button
            variant="outline"
            color="red"
            disabled={rows.length === 0}
            onClick={() => setRevokingAll(true)}
          >
            {t('customers.tokens.revokeAll')}
          </Button>
        )}
      </Flex>
      <DataTable
        label={t('customers.tokens.tableLabel')}
        columns={columns}
        rows={tokens.data}
        loading={tokens.isPending}
        error={tokens.error}
        onRetry={() => void tokens.refetch()}
        getRowId={(row) => row.id}
        rowActions={canRevoke ? rowActions : undefined}
        empty={
          <Text as="p" size="2" color="gray">
            {t('customers.tokens.empty')}
          </Text>
        }
      />
      <ConfirmDialog
        open={revoking !== null}
        onOpenChange={(open) => {
          if (!open) {
            setRevoking(null);
          }
        }}
        title={t('customers.tokens.revokeTitle')}
        description={revoking ? t('customers.tokens.revokeText', { name: revoking.name }) : ''}
        confirmLabel={t('customers.tokens.revokeConfirm')}
        danger
        onConfirm={revokeOne}
      />
      <ConfirmDialog
        open={revokingAll}
        onOpenChange={setRevokingAll}
        title={t('customers.tokens.revokeAllTitle')}
        description={t('customers.tokens.revokeAllText', { count: rows.length })}
        confirmLabel={t('customers.tokens.revokeAll')}
        danger
        onConfirm={revokeAll}
      />
    </Card>
  );
}
