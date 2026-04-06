import { Info, Shield, Users } from 'lucide-react';
import { useMemo } from 'react';
import { Link, useParams } from 'react-router';
import { path, useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import { PRIVILEGES, type Privilege, ROLES, type Role, USERS, type User } from '../api';
import { effectivePrivileges } from '../shared/effective';
import { PRIVILEGES_PAGE, privilegeHref, roleProfileHref, userHref } from '../shared/links';
import {
  ActionChips,
  LinkTable,
  OnlineStatus,
  ProfileActions,
  ProfileCard,
  ProfileRow,
  ProfileSection,
  ProfileTabs,
  Separator,
  SourceBadge,
} from '../shared/Profile';
import profile from '../shared/Profile.module.css';
import { actionLabel, typeLabel } from './labels';
import { permissionOf } from './model';

const REPOSITORY_ACTIONS = ['BROWSE', 'READ', 'EDIT', 'ADD', 'DELETE'] as const;

export function rolesWith(name: string, roles: readonly Role[]): Role[] {
  return roles.filter((role) => role.privileges.includes(name));
}

export function usersWith(name: string, users: readonly User[], roles: readonly Role[]): User[] {
  const byId = new Map(roles.map((role) => [role.id, role]));
  return users.filter((user) =>
    effectivePrivileges(user.roles, byId, new Map()).privileges.some((entry) => entry.name === name),
  );
}

export function fullName(user: User): string {
  return [user.firstName, user.lastName].filter(Boolean).join(' ');
}

function Overview({ privilege }: { privilege: Privilege }) {
  const t = useT();
  const permission = permissionOf(privilege);
  const actions = privilege.actions ?? [];
  const repositoryLike =
    privilege.type === 'repository-view' ||
    privilege.type === 'repository-admin' ||
    privilege.type === 'repository-content-selector';
  return (
    <div>
      <ProfileSection title={t('security.privilegeProfile.metadata')}>
        <ProfileRow label={t('security.privilegeField.name')}>{privilege.name}</ProfileRow>
        <ProfileRow label={t('security.privileges.type')}>{typeLabel(t, privilege.type)}</ProfileRow>
        <ProfileRow label={t('security.privilegeField.description')}>{privilege.description}</ProfileRow>
      </ProfileSection>
      <Separator />
      <ProfileSection title={t('security.privilegeProfile.configuration')}>
        {privilege.type === 'application' && (
          <>
            <ProfileRow label={t('security.privilegeField.domain')}>{privilege.domain}</ProfileRow>
            <ProfileRow label={t('security.privilegeField.actions')}>
              {actions.map((action) => actionLabel(t, action)).join(', ')}
            </ProfileRow>
          </>
        )}
        {privilege.type === 'wildcard' && (
          <ProfileRow label={t('security.privilegeField.pattern')}>{privilege.pattern}</ProfileRow>
        )}
        {privilege.type === 'repository-content-selector' && (
          <ProfileRow label={t('security.privilegeField.contentSelector')}>
            {privilege.contentSelector}
          </ProfileRow>
        )}
        {repositoryLike && (
          <>
            {privilege.type !== 'repository-content-selector' && (
              <ProfileRow label={t('security.privilegeField.format')}>{privilege.format}</ProfileRow>
            )}
            <ProfileRow label={t('security.privilegeField.repository')}>{privilege.repository}</ProfileRow>
            <ProfileRow label={t('security.privilegeField.actions')}>
              <ActionChips
                items={REPOSITORY_ACTIONS.map((action) => ({
                  label: actionLabel(t, action),
                  on: actions.includes('ALL') || actions.includes(action),
                }))}
              />
            </ProfileRow>
          </>
        )}
      </ProfileSection>
      {permission && (
        <>
          <Separator />
          <p className={profile.apiLink}>
            <Link to={`/api?permission=${encodeURIComponent(permission)}`}>
              {t('security.privilegeProfile.api')}
            </Link>
            <span className={profile.apiHint}>
              {t.rich('security.privilegeProfile.apiHint', { param: <code key="param">?permission=</code> })}
            </span>
          </p>
        </>
      )}
    </div>
  );
}

export default function PrivilegeProfilePage() {
  const t = useT();
  const { name = '' } = useParams();
  const canUpdate = useCan('astrum:privileges:update');
  const privilege = useApi<Privilege>(`${PRIVILEGES}${path`/${name}`}`);
  const roles = useApi<Role[]>(ROLES);
  const canUsers = useCan('astrum:users:read');
  const users = useApi<User[]>(USERS, { enabled: canUsers });

  const holders = useMemo(() => rolesWith(name, roles.data ?? []), [name, roles.data]);
  const reached = useMemo(
    () => usersWith(name, users.data ?? [], roles.data ?? []),
    [name, users.data, roles.data],
  );

  const data = privilege.data;
  return (
    <Page
      gutter="small"
      title={name}
      description={data ? typeLabel(t, data.type) : undefined}
      breadcrumbs={[{ label: t('security.privileges.title'), to: PRIVILEGES_PAGE }, { label: name }]}
      actions={
        <ProfileActions
          edit={canUpdate && data && !data.readOnly ? privilegeHref(name) : undefined}
          editLabel={t('security.privilegeProfile.edit')}
          back={PRIVILEGES_PAGE}
        />
      }
    >
      {privilege.error ? (
        <ErrorState error={privilege.error} onRetry={() => void privilege.refetch()} />
      ) : (
        data && (
          <ProfileTabs
            tabs={[
              {
                value: 'overview',
                label: t('security.privilegeProfile.overview'),
                icon: Info,
                content: <Overview privilege={data} />,
              },
              {
                value: 'roles',
                label: t('security.privilegeProfile.roles'),
                icon: Shield,
                content: (
                  <ProfileCard>
                    <LinkTable
                      label={t('security.privilegeProfile.rolesLabel')}
                      rows={holders}
                      getRowId={(role) => role.id}
                      empty={t('security.privilegeProfile.noRoles')}
                      columns={[
                        {
                          id: 'name',
                          header: t('security.privileges.name'),
                          cell: (role) => (
                            <Link
                              to={roleProfileHref(role.id)}
                              aria-label={t('security.privilegeProfile.openRole', { name: role.id })}
                            >
                              {role.name || role.id}
                            </Link>
                          ),
                        },
                        { id: 'id', header: t('security.roles.id'), cell: (role) => role.id },
                        {
                          id: 'source',
                          header: t('security.users.source'),
                          cell: (role) => (
                            <SourceBadge>{role.source === 'default' ? 'Default' : role.source}</SourceBadge>
                          ),
                        },
                        {
                          id: 'description',
                          header: t('security.privileges.descriptionColumn'),
                          cell: (role) => role.description,
                        },
                      ]}
                    />
                  </ProfileCard>
                ),
              },
              {
                value: 'users',
                label: t('security.privilegeProfile.users'),
                icon: Users,
                content: (
                  <ProfileCard>
                    <LinkTable
                      label={t('security.privilegeProfile.usersLabel')}
                      rows={reached}
                      getRowId={(user) => `${user.source}/${user.userId}`}
                      empty={t('security.privilegeProfile.noUsers')}
                      columns={[
                        {
                          id: 'id',
                          header: t('security.users.userId'),
                          cell: (user) => <Link to={userHref(user.userId, user.source)}>{user.userId}</Link>,
                        },
                        { id: 'name', header: t('security.privileges.name'), cell: fullName },
                        {
                          id: 'status',
                          header: t('security.users.status'),
                          cell: (user) => (
                            <OnlineStatus
                              active={user.status === 'active'}
                              label={
                                user.status === 'active'
                                  ? t('security.privilegeProfile.online')
                                  : t('security.privilegeProfile.offline')
                              }
                            />
                          ),
                        },
                        {
                          id: 'source',
                          header: t('security.users.source'),
                          cell: (user) =>
                            user.source === 'default' ? t('security.users.sourceLocal') : user.source,
                        },
                      ]}
                    />
                  </ProfileCard>
                ),
              },
            ]}
          />
        )
      )}
    </Page>
  );
}
