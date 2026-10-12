import { Callout, Flex, Text } from '@radix-ui/themes';
import { Info, KeyRound } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useApi } from '../../../api';
import { useT } from '../../../i18n';
import { type Column, DataTable, EmptyState, StatusBadge } from '../../../ui';
import { PRIVILEGES, type Privilege, ROLES, type Role } from '../api';
import { typeLabel } from '../privileges/labels';
import { permissionOf } from '../privileges/model';
import { type EffectivePrivilege, effectivePrivileges } from './effective';
import styles from './Form.module.css';
import { ListSearch } from './ListSearch';
import { privilegeHref } from './links';

export function EffectivePrivileges({ roleIds, label }: { roleIds: readonly string[]; label: string }) {
  const t = useT();
  const [query, setQuery] = useState('');
  const roles = useApi<Role[]>(ROLES);
  const privileges = useApi<Privilege[]>(PRIVILEGES);
  const effective = useMemo(() => {
    if (!roles.data || !privileges.data) {
      return undefined;
    }
    return effectivePrivileges(
      roleIds,
      new Map(roles.data.map((role) => [role.id, role])),
      new Map(privileges.data.map((privilege) => [privilege.name, privilege])),
    );
  }, [roleIds, roles.data, privileges.data]);
  const rows = useMemo(() => {
    const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return effective?.privileges.filter((entry) => {
      const text =
        `${entry.name} ${entry.privilege ? (permissionOf(entry.privilege) ?? '') : ''} ${entry.paths.flat().join(' ')}`.toLowerCase();
      return words.every((word) => text.includes(word));
    });
  }, [effective, query]);

  const columns: Column<EffectivePrivilege>[] = [
    {
      id: 'name',
      header: t('security.effective.privilege'),
      cell: (row) => row.name,
      sortValue: (row) => row.name,
      primary: true,
    },
    {
      id: 'type',
      header: t('security.privileges.type'),
      cell: (row) =>
        row.privilege ? (
          <StatusBadge tone="info">{typeLabel(t, row.privilege.type)}</StatusBadge>
        ) : (
          <StatusBadge tone="warning">{t('security.effective.missing')}</StatusBadge>
        ),
    },
    {
      id: 'permission',
      header: t('security.privileges.permission'),
      cell: (row) => (
        <span className={styles.mono}>{row.privilege ? (permissionOf(row.privilege) ?? '—') : '—'}</span>
      ),
      hideOnPhone: true,
    },
    {
      id: 'from',
      header: t('security.effective.from'),
      cell: (row) => (
        <Flex direction="column" gap="1">
          {row.paths.map((path) => (
            <Text key={path.join('/')} size="2">
              {path.join(' → ')}
            </Text>
          ))}
        </Flex>
      ),
    },
  ];

  return (
    <Flex direction="column" gap="3">
      <Text size="2" color="gray">
        {t('security.effective.intro')}
      </Text>
      {effective && effective.missingRoles.length > 0 && (
        <Callout.Root color="amber" variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>
            {t('security.effective.missingRoles', { roles: effective.missingRoles.join(', ') })}
          </Callout.Text>
        </Callout.Root>
      )}
      <div className={styles.toolbar}>
        <ListSearch value={query} onChange={setQuery} label={t('security.effective.search')} />
      </div>
      <DataTable
        label={label}
        columns={columns}
        rows={rows}
        getRowId={(row) => row.name}
        loading={roles.isPending || privileges.isPending}
        error={roles.error ?? privileges.error}
        rowHref={(row) => (row.privilege ? privilegeHref(row.name) : undefined)}
        initialSort={{ column: 'name', direction: 'asc' }}
        pageSize={50}
        empty={<EmptyState icon={KeyRound} title={t('security.effective.empty')} />}
      />
    </Flex>
  );
}
