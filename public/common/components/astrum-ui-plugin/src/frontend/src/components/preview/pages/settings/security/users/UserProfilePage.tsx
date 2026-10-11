

import React, { useState, useEffect, useMemo } from 'react';
import { Box, Flex, Text, Tabs, Badge, Card } from '@radix-ui/themes';
import { ExtJS } from '../../../../../../interface/ExtJS';
import {
  Shield,
  Key,
  Info,
  Pencil,
} from 'lucide-react';

import {
  LoadingState,
  ErrorState,
  EntityTable,
  EmptyState,
  TableColumn,
  MetadataGrid,
  PageHeader,
  StatusBadge,
} from '../../../../shared';
import { SettingsFormSection, SettingsButton } from '../../../../shared/form';
import { RoleExplorerTree } from '../roles/RoleExplorerTree';
import { useSecurityEntityModal } from '../SecurityEntityModalContext';
import { useUserTree } from './useUserTree';
import { useUserEffectivePrivileges, type EffectivePrivilege } from './useUserEffectivePrivileges';
import { useUsersApi } from './useUsersApi';
import {
  User,
  DEFAULT_SOURCE,
  getSourceLabel,
} from './types';
import { Role } from '../roles/types';
import { apiHubHref } from '../apiHubLinks';

import './UserProfilePage.scss';
import UsersStrings from '../../../../constants/pages/settings/security/UsersStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface UserProfilePageProps {
  userId: string;
  userSource: string;
  onBack: () => void;
  onEdit?: () => void;
  canEdit?: boolean;
  /** When true, cross-entity links (roles, privileges) render as plain text — used in full-screen modal */
  embedMode?: boolean;
}

const ROLES_BASE = 'preview/admin/security/roles';
const PRIVILEGES_BASE = 'preview/admin/security/privileges';

function _roleProfileUrl(roleId: string): string {
  return `#${ROLES_BASE}/${encodeURIComponent(roleId)}/profile`;
}

function _privilegeProfileUrl(privilegeId: string): string {
  return `#${PRIVILEGES_BASE}/${encodeURIComponent(privilegeId)}/profile`;
}

/**
 * UserProfilePage - Read-only operational dashboard for a User.
 * Tabs: Overview, Roles, Privileges, Security Tree.
 * Follows Role Profile pattern.
 */
export function UserProfilePage({
  userId,
  userSource,
  onBack,
  onEdit,
  canEdit = false,
  embedMode = false,
}: UserProfilePageProps) {
  const [user, setUser] = useState<User | null>(null);
  const [userLoading, setUserLoading] = useState(true);
  const [userError, setUserError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('overview');

  const { fetchUser } = useUsersApi();
  const {
    tree,
    loading: treeLoading,
    error: _treeError,
    toggleExpand,
    expandAll,
    collapseAll,
    setSearchTerm,
  } = useUserTree(user);
  const {
    privileges,
    roleMap,
    loading: privilegesLoading,
  } = useUserEffectivePrivileges(user);

  useEffect(() => {
    const load = async () => {
      setUserLoading(true);
      setUserError(null);
      try {
        const data = await fetchUser(userId, userSource || DEFAULT_SOURCE);
        setUser(data);
      } catch (err: unknown) {
        setUserError(err instanceof Error ? err.message : UsersStrings.PROFILE.FAILED_LOAD_USER);
      } finally {
        setUserLoading(false);
      }
    };
    load();
  }, [userId, userSource, fetchUser]);

  const { openEntity } = useSecurityEntityModal();
  const canLinkApiHub = ExtJS.checkPermission('astrum:settings:read');

  const rolesTableData = useMemo(() => {
    if (!user?.roles) return [];
    return user.roles
      .map((roleId) => roleMap.get(roleId))
      .filter((r): r is Role => !!r);
  }, [user?.roles, roleMap]);

  const roleColumns: TableColumn<Role>[] = useMemo(
    () => [
      {
        id: 'name',
        header: CommonStrings.LABELS.NAME,
        accessor: (role) =>
          embedMode ? (
            <Text size="2" as="span">{role.name}</Text>
          ) : (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                openEntity('role', role.id);
              }}
              style={{
                background: 'none',
                border: 'none',
                padding: 0,
                font: 'inherit',
                color: 'var(--blue-11)',
                textDecoration: 'underline',
                cursor: 'pointer',
              }}
              aria-label={UsersStrings.PROFILE.OPEN_ROLE_PROFILE(role.name)}
            >
              <Text size="2" as="span">{role.name}</Text>
            </button>
          ),
        sortable: true,
      },
      {
        id: 'id',
        header: 'ID',
        accessor: (role) => <Text size="2">{role.id}</Text>,
        sortable: true,
      },
      {
        id: 'source',
        header: UsersStrings.PROFILE.SOURCE,
        accessor: (role) => (
          <Badge color="blue" variant="soft">
            {role.source || UsersStrings.PROFILE.DEFAULT}
          </Badge>
        ),
        sortable: true,
      },
      {
        id: 'description',
        header: CommonStrings.LABELS.DESCRIPTION,
        accessor: (role) => <Text size="2">{role.description || '—'}</Text>,
        sortable: false,
      },
    ],
    [embedMode, openEntity]
  );

  const privilegeColumns: TableColumn<EffectivePrivilege>[] = useMemo(
    () => [
      {
        id: 'name',
        header: CommonStrings.LABELS.NAME,
        accessor: (p) => (
          <Flex align="center" gap="2">
            <Key size={14} color="var(--amber-9)" aria-hidden="true" />
            {embedMode ? (
              <Text size="2" weight="medium" as="span">{p.name}</Text>
            ) : (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  openEntity('privilege', p.id);
                }}
                style={{
                  background: 'none',
                  border: 'none',
                  padding: 0,
                  font: 'inherit',
                  color: 'var(--blue-11)',
                  textDecoration: 'underline',
                  cursor: 'pointer',
                }}
                aria-label={UsersStrings.PROFILE.OPEN_PRIVILEGE_PROFILE(p.name)}
              >
                <Text size="2" weight="medium" as="span">{p.name}</Text>
              </button>
            )}
          </Flex>
        ),
        sortable: true,
      },
      {
        id: 'permission',
        header: UsersStrings.PROFILE.PERMISSION,
        accessor: (p) => (
          <Text size="1" color="gray" style={{ fontFamily: 'var(--font-mono)' }}>
            {p.permission}
          </Text>
        ),
        sortable: true,
      },
      {
        id: 'type',
        header: CommonStrings.LABELS.TYPE,
        accessor: (p) => <Text size="2">{p.type || '—'}</Text>,
        sortable: true,
      },
      {
        id: 'repository',
        header: UsersStrings.PROFILE.REPOSITORY,
        accessor: (p) => (
          <Text size="2">{p.properties?.repository || '—'}</Text>
        ),
        sortable: true,
      },
      {
        id: 'actions',
        header: CommonStrings.LABELS.ACTIONS,
        accessor: (p) => (
          <Text size="2">{p.properties?.actions || '—'}</Text>
        ),
        sortable: true,
      },
      {
        id: 'grantedBy',
        header: UsersStrings.PROFILE.GRANTED_BY,
        accessor: (p) => (
          <Flex wrap="wrap" gap="1">
            {(p.grantedByRoleIds || []).map((roleId, idx) => {
              const role = roleMap.get(roleId);
              const label = role?.name || roleId;
              const isLast = idx === (p.grantedByRoleIds?.length ?? 1) - 1;
              return (
                <React.Fragment key={roleId}>
                  {embedMode ? (
                    <Text size="1" as="span">{label}</Text>
                  ) : (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        openEntity('role', roleId);
                      }}
                      style={{
                        background: 'none',
                        border: 'none',
                        padding: 0,
                        font: 'inherit',
                        color: 'var(--blue-11)',
                        textDecoration: 'underline',
                        fontSize: 'var(--font-size-1)',
                        cursor: 'pointer',
                      }}
                      aria-label={UsersStrings.PROFILE.OPEN_ROLE_PROFILE(label)}
                    >
                      {label}
                    </button>
                  )}
                  {!isLast && <Text size="1">{', '}</Text>}
                </React.Fragment>
              );
            })}
          </Flex>
        ),
        sortable: false,
      },
    ],
    [embedMode, openEntity, roleMap]
  );

  if (userLoading) return <LoadingState message={UsersStrings.PROFILE.LOADING_USER_PROFILE} />;
  if (userError) return <ErrorState message={userError} onRetry={() => window.location.reload()} />;
  if (!user) return <ErrorState message={UsersStrings.PROFILE.USER_NOT_FOUND} />;

  const sourceLabel = getSourceLabel(user.source || DEFAULT_SOURCE);
  const isActive = user.status === 'active';
  const statusLabel = isActive ? UsersStrings.PROFILE.ACTIVE : UsersStrings.PROFILE.DISABLED;

  return (
    <Box className="user-profile-page" data-testid="user-profile-page">
      <PageHeader
        title={user.userId}
        headingAs={embedMode ? 'h2' : 'h1'}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#preview/admin/settings'; } },
          { label: UsersStrings.PROFILE.USERS, onClick: onBack },
          { label: user.userId }
        ]}
        actions={!embedMode ? (
          <Flex gap="2">
            {canEdit && onEdit && (
              <SettingsButton variant="primary" onClick={onEdit} icon={Pencil}>
                {UsersStrings.PROFILE.EDIT_USER}
              </SettingsButton>
            )}
          </Flex>
        ) : undefined}
        meta={
          <>
            <StatusBadge
              status={isActive ? 'online' : 'offline'}
              size="small"
              label={statusLabel}
            />
            <Badge color="blue" variant="soft">
              {sourceLabel}
            </Badge>
          </>
        }
      />

      <Tabs.Root value={activeTab} onValueChange={setActiveTab}>
        <Tabs.List className="user-profile-page__tabs-list">
          <Tabs.Trigger value="overview">
            <Flex align="center" gap="1">
              <Info size={14} aria-hidden="true" />
              <Text size="2">{UsersStrings.PROFILE.OVERVIEW}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="roles">
            <Flex align="center" gap="1">
              <Shield size={14} aria-hidden="true" />
              <Text size="2">{UsersStrings.PROFILE.ROLES}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="privileges">
            <Flex align="center" gap="1">
              <Key size={14} aria-hidden="true" />
              <Text size="2">{UsersStrings.PROFILE.PRIVILEGES}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="tree">
            <Flex align="center" gap="1">
              <Shield size={14} aria-hidden="true" />
              <Text size="2">{UsersStrings.PROFILE.SECURITY_TREE}</Text>
            </Flex>
          </Tabs.Trigger>
        </Tabs.List>

        <Box pt="4" className="user-profile-page__content">
          <Tabs.Content value="overview">
            <SettingsFormSection title={UsersStrings.PROFILE.USER_METADATA} defaultOpen>
              <MetadataGrid
                items={[
                  {
                    label: UsersStrings.PROFILE.USER_ID,
                    value: (
                      <Text size="2" style={{ fontFamily: 'var(--font-mono)' }}>
                        {user.userId}
                      </Text>
                    ),
                  },
                  { label: UsersStrings.PROFILE.FIRST_NAME, value: user.firstName },
                  { label: UsersStrings.PROFILE.LAST_NAME, value: user.lastName },
                  { label: UsersStrings.PROFILE.EMAIL, value: user.emailAddress || user.email },
                  { label: CommonStrings.LABELS.STATUS, value: statusLabel },
                  {
                    label: UsersStrings.PROFILE.SOURCE,
                    value: (
                      <Badge color="blue" variant="soft">
                        {sourceLabel}
                      </Badge>
                    ),
                  },
                ]}
              />
            </SettingsFormSection>
          </Tabs.Content>

          <Tabs.Content value="roles">
            <Box className="user-profile-page__pane">
              <EntityTable<Role>
                data={rolesTableData}
                columns={roleColumns}
                getRowKey={(r) => r.id}
                loading={privilegesLoading && user.roles!.length > 0}
                loadingMessage={UsersStrings.PROFILE.LOADING_ROLES}
                ariaLabel={UsersStrings.PROFILE.ROLES_ASSIGNED_USER_ARIA}
                emptyState={
                  <EmptyState
                    title={UsersStrings.PROFILE.NO_ROLES_ASSIGNED}
                    description={UsersStrings.PROFILE.USER_HAS_NO_ROLES}
                    icon={Shield}
                  />
                }
              />
            </Box>
          </Tabs.Content>

          <Tabs.Content value="privileges">
            <Box className="user-profile-page__pane">
              {canLinkApiHub && !embedMode && (
                <Box mb="3">
                  <Text size="2">
                    <a href={apiHubHref({ user: userId })} className="user-profile-page__api-hub-link">
                      {UsersStrings.PROFILE.VIEW_API_ACCESS_USER}
                    </a>{' '}
                    <Text size="1" color="gray" as="span">
                      {UsersStrings.PROFILE.OPENS_API_HUB(<Text as="span" style={{ fontFamily: 'var(--font-mono)' }}>?user=</Text>)}
                    </Text>
                  </Text>
                </Box>
              )}
              <EntityTable<EffectivePrivilege>
                data={privileges}
                columns={privilegeColumns}
                getRowKey={(p) => p.id}
                loading={privilegesLoading}
                loadingMessage={UsersStrings.PROFILE.CALCULATING_EFFECTIVE_PRIVILEGES}
                ariaLabel={UsersStrings.PROFILE.EFFECTIVE_PRIVILEGES_ARIA}
                emptyState={
                  <EmptyState
                    title={UsersStrings.PROFILE.NO_PRIVILEGES}
                    description={UsersStrings.PROFILE.USER_HAS_NO_EFFECTIVE_PRIVILEGES}
                    icon={Key}
                  />
                }
              />
            </Box>
          </Tabs.Content>

          <Tabs.Content value="tree">
            <Card className="user-profile-page__pane user-profile-page__pane--tree">
              {user.roles?.length ? (
                <RoleExplorerTree
                  tree={tree}
                  loading={treeLoading}
                  onToggleExpand={toggleExpand}
                  onExpandAll={expandAll}
                  onCollapseAll={collapseAll}
                  onSearchChange={setSearchTerm}
                />
              ) : (
                <Box className="user-profile-page__empty-tree">
                  <Text size="2" color="gray">
                    {UsersStrings.PROFILE.USER_HAS_NO_ROLES_ASSIGNED}
                  </Text>
                </Box>
              )}
            </Card>
          </Tabs.Content>
        </Box>
      </Tabs.Root>
    </Box>
  );
}

export default UserProfilePage;
