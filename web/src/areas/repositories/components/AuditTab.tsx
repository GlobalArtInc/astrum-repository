import { Button, Flex, Heading } from '@radix-ui/themes';
import { History, ScrollText } from 'lucide-react';
import { Link } from 'react-router';
import { useApi, withQuery } from '../../../api';
import { type Translate, useT } from '../../../i18n';
import { type Column, DataTable, DateTime, EmptyState } from '../../../ui';
import { type AuditEvent, auditPath } from '../api';

export const AUDIT_LIMIT = 20;

const KINDS: Record<string, string> = {
  created: 'repositories.audit.kinds.created',
  updated: 'repositories.audit.kinds.updated',
  deleted: 'repositories.audit.kinds.deleted',
};

export function eventLabel(t: Translate, kind: string): string {
  const key = KINDS[kind];
  return key ? t(key) : kind;
}

function columns(t: Translate): Column<AuditEvent>[] {
  return [
    {
      id: 'time',
      header: t('repositories.audit.time'),
      cell: (row) => <DateTime value={row.at} />,
      primary: true,
    },
    { id: 'event', header: t('repositories.audit.event'), cell: (row) => eventLabel(t, row.type) },
    {
      id: 'object',
      header: t('repositories.audit.object'),
      cell: (row) => (row.context ? `${row.domain}: ${row.context}` : row.domain),
    },
    { id: 'initiator', header: t('repositories.audit.initiator'), cell: (row) => row.initiator },
  ];
}

export function auditLogAddress(name: string): string {
  return withQuery('/audit', { repositoryName: name });
}

export function AuditTab({ name }: { name: string }) {
  const t = useT();
  const query = useApi<AuditEvent[]>(auditPath(name, AUDIT_LIMIT));
  return (
    <Flex direction="column" gap="3">
      <Flex justify="between" align="center" gap="3" wrap="wrap">
        <Heading as="h2" size="4">
          {t('repositories.audit.title')}
        </Heading>
        <Button asChild variant="soft">
          <Link to={auditLogAddress(name)}>
            <ScrollText size={16} aria-hidden="true" />
            {t('repositories.audit.openLog')}
          </Link>
        </Button>
      </Flex>
      <DataTable
        label={t('repositories.audit.label')}
        columns={columns(t)}
        rows={query.data}
        getRowId={(row) => String(row.id)}
        loading={query.isPending}
        error={query.error}
        onRetry={() => void query.refetch()}
        empty={<EmptyState icon={History} title={t('repositories.audit.empty')} />}
      />
    </Flex>
  );
}
