import { Button, Flex, Spinner } from '@radix-ui/themes';
import { ArrowLeft, Info, Key, Pencil, Shield, User as UserIcon, Users } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { path, useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import {
  type ContentSelector,
  PRIVILEGES,
  type Privilege,
  ROLES,
  type Role,
  SELECTORS,
  USERS,
  type User,
} from '../api';
import { permissionOf } from '../privileges/model';
import { type ListColumn, ListTable, SearchBox, sortList, useCollator } from '../shared/AdminList';
import { effectivePrivileges } from '../shared/effective';
import { privilegeProfileHref, ROLES_PAGE, roleHref, userHref } from '../shared/links';
import { ProfileRow, ProfileSection, ProfileTabs } from '../shared/Profile';
import { buildTree, SecurityTree } from '../shared/SecurityTree';
import styles from './Roles.module.css';

function useMaps(enabled: boolean) {
  const canSelectors = useCan('astrum:selectors:read');
  const roles = useApi<Role[]>(enabled ? ROLES : null);
  const privileges = useApi<Privilege[]>(enabled ? PRIVILEGES : null);
  const selectors = useApi<ContentSelector[]>(enabled && canSelectors ? SELECTORS : null);
  return useMemo(
    () => ({
      ready: roles.data !== undefined && privileges.data !== undefined,
      roles: new Map((roles.data ?? []).map((role) => [role.id, role])),
      privileges: new Map((privileges.data ?? []).map((privilege) => [privilege.name, privilege])),
      selectors: new Map((selectors.data ?? []).map((selector) => [selector.name, selector])),
    }),
    [roles.data, privileges.data, selectors.data],
  );
}

export function RoleTree({ roots, label }: { roots: readonly string[]; label: string }) {
  const maps = useMaps(true);
  const nodes = useMemo(() => buildTree(roots, maps.roles, maps.privileges, maps.selectors), [roots, maps]);
  if (!maps.ready) {
    return (
      <Flex justify="center" py="6">
        <Spinner />
      </Flex>
    );
  }
  return <SecurityTree key={roots.join('|')} label={label} nodes={nodes} />;
}

function Effective({ id }: { id: string }) {
  const t = useT();
  const collator = useCollator();
  const maps = useMaps(true);
  const [query, setQuery] = useState('');
  const [direction, setDirection] = useState<'asc' | 'desc'>('asc');
  const entries = useMemo(
    () =>
      effectivePrivileges([id], maps.roles, maps.privileges)
        .privileges.filter((entry) => entry.privilege)
        .map((entry) => ({
          name: entry.name,
          permission: entry.privilege ? (permissionOf(entry.privilege) ?? '') : '',
        })),
    [id, maps],
  );
  const words = query.trim().toLowerCase();
  const rows = sortList(
    entries.filter((entry) => !words || `${entry.name} ${entry.permission}`.toLowerCase().includes(words)),
    (entry) => entry.name,
    direction,
    collator,
  );
  const columns: ListColumn<{ name: string; permission: string }>[] = [
    {
      id: 'name',
      header: t('security.privileges.name'),
      sortValue: (row) => row.name,
      cell: (row) => (
        <span className={styles.linkCell}>
          <Key size={14} aria-hidden="true" className={styles.key} />
          <Link to={privilegeProfileHref(row.name)} className={styles.link}>
            {row.name}
          </Link>
        </span>
      ),
    },
    {
      id: 'permission',
      header: t('security.privileges.permission'),
      cell: (row) => <span className={styles.permission}>{row.permission}</span>,
    },
  ];
  return (
    <section className={styles.card} aria-label={t('security.roleProfile.effectiveTitle')}>
      <h3 className={styles.cardTitle}>{t('security.roleProfile.effectiveTitle')}</h3>
      <p className={styles.cardText}>{t('security.roleProfile.effectiveIntro')}</p>
      <SearchBox value={query} onChange={setQuery} placeholder={t('security.roleProfile.effectiveSearch')} />
      <ListTable
        label={t('security.roleProfile.effectiveTitle')}
        columns={columns}
        rows={rows}
        getRowId={(row) => row.name}
        loading={!maps.ready}
        sort={{ column: 'name', direction }}
        onSort={() => setDirection(direction === 'asc' ? 'desc' : 'asc')}
        empty={<p className={styles.intro}>{t('security.roleProfile.effectiveEmpty')}</p>}
      />
    </section>
  );
}

function Assigned({ id }: { id: string }) {
  const t = useT();
  const navigate = useNavigate();
  const canUsers = useCan('astrum:users:read');
  const users = useApi<User[]>(canUsers ? USERS : null);
  const rows = (users.data ?? []).filter((user) => user.roles.includes(id));
  const columns: ListColumn<User>[] = [
    {
      id: 'id',
      header: t('security.users.userId'),
      cell: (row) => (
        <span className={styles.linkCell}>
          <UserIcon size={14} aria-hidden="true" className={styles.person} />
          <span className={styles.userId}>{row.userId}</span>
        </span>
      ),
    },
    {
      id: 'name',
      header: t('security.users.name'),
      cell: (row) => [row.firstName, row.lastName].filter(Boolean).join(' '),
    },
    { id: 'email', header: t('security.users.email'), cell: (row) => row.emailAddress },
    { id: 'source', header: t('security.users.source'), cell: (row) => row.source },
  ];
  return (
    <section className={styles.card} aria-label={t('security.roleProfile.users')}>
      <p className={styles.note}>
        <Info size={14} aria-hidden="true" />
        <span>{t.rich('security.roleProfile.usersIntro', { id: <strong key="id">{id}</strong> })}</span>
      </p>
      <div className={styles.usersTable}>
        <ListTable
          label={t('security.roleProfile.users')}
          columns={columns}
          rows={rows}
          getRowId={(row) => `${row.source}/${row.userId}`}
          loading={users.isPending && canUsers}
          rowLabel={(row) => row.userId}
          onOpen={(row) => navigate(userHref(row.userId, row.source))}
          empty={<p className={styles.intro}>{t('security.roleProfile.usersEmpty')}</p>}
        />
      </div>
    </section>
  );
}

export default function RoleProfilePage() {
  const t = useT();
  const { id = '' } = useParams();
  const canUpdate = useCan('astrum:roles:update');
  const role = useApi<Role>(`${ROLES}${path`/${id}`}`);
  const data = role.data;
  const roots = useMemo(() => [id], [id]);
  return (
    <Page
      gutter="small"
      title={id}
      description={t('security.roleProfile.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.roles.title'), to: ROLES_PAGE },
        { label: id },
      ]}
      actions={
        <>
          {canUpdate && data && !data.readOnly && (
            <Button asChild>
              <Link to={roleHref(id)}>
                <Pencil size={16} aria-hidden="true" />
                {t('security.roleProfile.edit')}
              </Link>
            </Button>
          )}
          <Button asChild variant="soft" color="gray">
            <Link to={ROLES_PAGE}>
              <ArrowLeft size={16} aria-hidden="true" />
              {t('security.profile.back')}
            </Link>
          </Button>
        </>
      }
    >
      {role.error ? (
        <ErrorState error={role.error} onRetry={() => void role.refetch()} />
      ) : (
        data && (
          <ProfileTabs
            tabs={[
              {
                value: 'overview',
                label: t('security.roleProfile.overview'),
                icon: Info,
                content: (
                  <ProfileSection title={t('security.roleProfile.metadata')}>
                    <ProfileRow label={t('security.role.id')}>
                      <span className={styles.mono}>{data.id}</span>
                    </ProfileRow>
                    <ProfileRow label={t('security.roles.name')}>{data.name}</ProfileRow>
                    <ProfileRow label={t('security.roles.descriptionColumn')}>{data.description}</ProfileRow>
                  </ProfileSection>
                ),
              },
              {
                value: 'tree',
                label: t('security.roleProfile.tree'),
                icon: Shield,
                content: <RoleTree roots={roots} label={t('security.roleProfile.treeLabel')} />,
              },
              {
                value: 'effective',
                label: t('security.roleProfile.effective'),
                icon: Key,
                content: <Effective id={id} />,
              },
              {
                value: 'users',
                label: t('security.roleProfile.users'),
                icon: Users,
                content: <Assigned id={id} />,
              },
            ]}
          />
        )
      )}
    </Page>
  );
}
