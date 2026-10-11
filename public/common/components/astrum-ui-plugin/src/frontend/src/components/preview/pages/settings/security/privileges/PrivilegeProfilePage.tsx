

import React, { useState, useMemo } from 'react';
import { Box, Flex, Text, Tabs, Badge } from '@radix-ui/themes';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { Key, Shield, Users, Info, ArrowLeft, Pencil, Check } from 'lucide-react';

import {
  PageHeader,
  LoadingState,
  ErrorState,
  EntityTable,
  EmptyState,
  TableColumn,
  StatusBadge,
  MetadataGrid,
} from '../../../../shared';
import { SettingsFormSection, SettingsButton } from '../../../../shared/form';
import { usePrivilegeProfile } from './usePrivilegeProfile';
import { getPrivilegeTypeLabel, getActionsForPrivilegeType, PRIVILEGE_TYPES } from './types';
import { Role, formatRoleSourceDisplay } from '../roles/types';
import { User, DEFAULT_SOURCE, getSourceLabel, getFullName } from '../users/types';
import { apiHubHref } from '../apiHubLinks';

import './PrivilegeProfilePage.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import PrivilegesStrings from '../../../../constants/pages/settings/security/PrivilegesStrings';

const ROLES_BASE_PATH = 'admin/security/roles';
const USERS_BASE_PATH = 'admin/security/users';

/**
 * ASTRUM-52167: these used to be styled <button>s calling openEntity() from
 * SecurityEntityModalContext, whose provider is never mounted — and the hook
 * falls back to a no-op instead of throwing, so the dead click was invisible.
 * Real anchors also restore middle-click, Cmd/Ctrl-click and "open in new tab".
 */
function roleProfileHash(roleId: string): string {
  return `#${ROLES_BASE_PATH}/${encodeURIComponent(roleId)}/profile`;
}

function userProfileHash(userId: string, source: string): string {
  return `#${USERS_BASE_PATH}/${encodeURIComponent(userId)}/${encodeURIComponent(source)}/profile`;
}

interface PrivilegeProfilePageProps {
  privilegeId: string;
  onBack: () => void;
  onEdit?: () => void;
  canEdit?: boolean;
  /** When true, cross-entity links (roles, users) render as plain text — used in full-screen modal */
  embedMode?: boolean;
  /**
   * Active tab, when the parent owns it via the URL. Omit to let the page manage
   * its own tab state (embedded/standalone use).
   */
  activeTab?: string;
  /** Called with the new tab id when the parent owns tab state. */
  onTabChange?: (tab: string) => void;
}

/**
 * PrivilegeProfilePage - Read-only profile for a Privilege.
 * Tabs: Overview, Roles Using This, Users With Access.
 * Per SECURITY-CROSS-NAVIGATION-DESIGN: cross-module links open in new tab.
 */
export function PrivilegeProfilePage({
  privilegeId,
  onBack,
  onEdit,
  canEdit = false,
  embedMode = false,
  activeTab: controlledTab,
  onTabChange,
}: PrivilegeProfilePageProps) {
  // Controlled when the parent supplies the tab (URL-driven), uncontrolled
  // otherwise (embedded/standalone).
  const [uncontrolledTab, setUncontrolledTab] = useState('overview');
  const activeTab = controlledTab ?? uncontrolledTab;

  const handleTabChange = (tab: string) => {
    if (onTabChange) {
      onTabChange(tab);
    } else {
      setUncontrolledTab(tab);
    }
  };

  const { privilege, rolesUsing, usersWithAccess, loading, error } =
    usePrivilegeProfile(privilegeId);

  const roleColumns: TableColumn<Role>[] = useMemo(
    () => [
      {
        id: 'name',
        header: CommonStrings.LABELS.NAME,
        accessor: (role) =>
          embedMode ? (
            <Text size="2" weight="medium" as="span">{role.name}</Text>
          ) : (
            <a
              href={roleProfileHash(role.id)}
              className="privilege-profile-page__entity-link"
              onClick={(e) => e.stopPropagation()}
              aria-label={PrivilegesStrings.PROFILE.OPEN_ROLE_PROFILE(role.name)}
            >
              <Text size="2" weight="medium" as="span">{role.name}</Text>
            </a>
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
        header: PrivilegesStrings.PROFILE.SOURCE,
        accessor: (role) => (
          <Badge color="blue" variant="soft">
            {formatRoleSourceDisplay(role.source)}
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
    [embedMode]
  );

  const userColumns: TableColumn<User>[] = useMemo(
    () => [
      {
        id: 'userId',
        header: PrivilegesStrings.PROFILE.USER_ID,
        accessor: (user) =>
          embedMode ? (
            <Text size="2" weight="medium" as="span">{user.userId}</Text>
          ) : (
            <a
              href={userProfileHash(user.userId, user.source || DEFAULT_SOURCE)}
              className="privilege-profile-page__entity-link"
              onClick={(e) => e.stopPropagation()}
              aria-label={PrivilegesStrings.PROFILE.OPEN_USER_PROFILE(user.userId)}
            >
              <Text size="2" weight="medium" as="span">{user.userId}</Text>
            </a>
          ),
        sortable: true,
      },
      {
        id: 'name',
        header: CommonStrings.LABELS.NAME,
        // ASTRUM-52167: plain text — User ID is the only navigation affordance.
        accessor: (user) => <Text size="2">{getFullName(user)}</Text>,
        sortable: true,
      },
      {
        id: 'status',
        header: CommonStrings.LABELS.STATUS,
        accessor: (user) => (
          <StatusBadge
            status={user.status === 'active' ? 'online' : 'offline'}
            size="small"
          />
        ),
        sortable: true,
      },
      {
        id: 'source',
        header: PrivilegesStrings.PROFILE.SOURCE,
        accessor: (user) => (
          <Text size="2">{getSourceLabel(user.source || DEFAULT_SOURCE)}</Text>
        ),
        sortable: true,
      },
    ],
    [embedMode]
  );

  if (loading && !privilege) return <LoadingState message={PrivilegesStrings.PROFILE.LOADING_PRIVILEGE_PROFILE} />;
  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;
  if (!privilege) return <ErrorState message={PrivilegesStrings.PROFILE.PRIVILEGE_NOT_FOUND} />;

  const canLinkApiHub = ExtJS.checkPermission('astrum:settings:read');

  return (
    <Box className="privilege-profile-page" data-testid="privilege-profile-page">
      <PageHeader
        icon={Key}
        title={privilege.name}
        description={`${getPrivilegeTypeLabel(privilege.type)}${privilege.readOnly ? PrivilegesStrings.PROFILE.READ_ONLY_SUFFIX : ''}`}
        breadcrumbs={[
          { label: PrivilegesStrings.PROFILE.PRIVILEGES, onClick: onBack },
          { label: privilege.name },
        ]}
        actions={
          !embedMode ? (
            <Flex gap="2">
              {canEdit && onEdit && (
                <SettingsButton variant="primary" onClick={onEdit} icon={Pencil}>
                  {PrivilegesStrings.PROFILE.EDIT_PRIVILEGE}
                </SettingsButton>
              )}
              <SettingsButton variant="ghost" onClick={onBack} icon={ArrowLeft}>
                {CommonStrings.ACTIONS.BACK_TO_LIST}
              </SettingsButton>
            </Flex>
          ) : undefined
        }
      />

      <Tabs.Root value={activeTab} onValueChange={handleTabChange}>
        <Tabs.List className="privilege-profile-page__tabs-list">
          <Tabs.Trigger value="overview">
            <Flex align="center" gap="1">
              <Info size={14} />
              <Text size="2">{PrivilegesStrings.PROFILE.OVERVIEW}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="roles">
            <Flex align="center" gap="1">
              <Shield size={14} />
              <Text size="2">{PrivilegesStrings.PROFILE.ROLES_USING}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="users">
            <Flex align="center" gap="1">
              <Users size={14} />
              <Text size="2">{PrivilegesStrings.PROFILE.USERS_ACCESS}</Text>
            </Flex>
          </Tabs.Trigger>
        </Tabs.List>

        <Box pt="4" className="privilege-profile-page__content">
          <Tabs.Content value="overview">
            <SettingsFormSection title={PrivilegesStrings.PROFILE.PRIVILEGE_METADATA} defaultOpen>
              <MetadataGrid
                items={[
                  { label: CommonStrings.LABELS.NAME, value: privilege.name },
                  { label: CommonStrings.LABELS.TYPE, value: getPrivilegeTypeLabel(privilege.type) },
                  { label: CommonStrings.LABELS.DESCRIPTION, value: privilege.description || '—' },
                  ...(privilege.readOnly
                    ? [
                        {
                          label: PrivilegesStrings.PROFILE.READ_ONLY_LABEL,
                          value: (
                            <Badge color="gray" variant="soft">
                              {PrivilegesStrings.PROFILE.READ_ONLY_SYSTEM}
                            </Badge>
                          ),
                        },
                      ]
                    : []),
                ]}
              />
            </SettingsFormSection>
            <SettingsFormSection title={PrivilegesStrings.PROFILE.CONFIGURATION} defaultOpen>
              <MetadataGrid
                items={[
                  // Type-specific fields
                  ...(privilege.type === PRIVILEGE_TYPES.WILDCARD
                    ? [
                        {
                          label: PrivilegesStrings.PROFILE.PRIVILEGE_STRING,
                          value: (
                            <Text size="2" style={{ fontFamily: 'var(--font-mono)' }}>
                              {privilege.properties?.pattern || '—'}
                            </Text>
                          ),
                        },
                      ]
                    : []),
                  ...(privilege.type === PRIVILEGE_TYPES.APPLICATION
                    ? [
                        { label: PrivilegesStrings.PROFILE.DOMAIN, value: privilege.properties?.domain || '—' },
                      ]
                    : []),
                  ...(privilege.type === PRIVILEGE_TYPES.REPOSITORY_VIEW ||
                  privilege.type === PRIVILEGE_TYPES.REPOSITORY_ADMIN
                    ? [
                        { label: CommonStrings.LABELS.FORMAT, value: privilege.properties?.format || '—' },
                        { label: PrivilegesStrings.PROFILE.REPOSITORY, value: privilege.properties?.repository || '—' },
                      ]
                    : []),
                  ...(privilege.type === PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR
                    ? [
                        { label: PrivilegesStrings.PROFILE.CONTENT_SELECTOR, value: privilege.properties?.contentSelector || '—' },
                        { label: PrivilegesStrings.PROFILE.REPOSITORY, value: privilege.properties?.repository || '—' },
                      ]
                    : []),
                  ...(privilege.type === PRIVILEGE_TYPES.SCRIPT
                    ? [
                        { label: PrivilegesStrings.PROFILE.SCRIPT_NAME, value: privilege.properties?.name || '—' },
                      ]
                    : []),
                  // Actions (if applicable)
                  ...(privilege.properties?.actions
                    ? [
                        {
                          label: CommonStrings.LABELS.ACTIONS,
                          value: (() => {
                            const actions = privilege.properties.actions.split(',').map((a) => a.trim());
                            const availableActions = getActionsForPrivilegeType(privilege.type);
                            if (availableActions.length === 0) return '—';

                            return (
                              <Flex gap="2" wrap="wrap">
                                {availableActions.map((option) => {
                                  const isChecked = actions.includes(option.value);
                                  return (
                                    <Flex
                                      key={option.value}
                                      align="center"
                                      gap="2"
                                      style={{
                                        opacity: isChecked ? 1 : 0.4,
                                        padding: '4px 8px',
                                        borderRadius: '4px',
                                        backgroundColor: isChecked ? 'var(--accent-3)' : 'var(--gray-3)',
                                      }}
                                    >
                                      <Box
                                        style={{
                                          width: '16px',
                                          height: '16px',
                                          borderRadius: '3px',
                                          border: '1px solid var(--gray-7)',
                                          backgroundColor: isChecked ? 'var(--accent-9)' : 'transparent',
                                          display: 'flex',
                                          alignItems: 'center',
                                          justifyContent: 'center',
                                        }}
                                      >
                                        {isChecked && <Check size={12} color="var(--accent-contrast)" />}
                                      </Box>
                                      <Text size="2">{option.label}</Text>
                                    </Flex>
                                  );
                                })}
                              </Flex>
                            );
                          })(),
                        },
                      ]
                    : []),
                ]}
              />
            </SettingsFormSection>
            {canLinkApiHub && !embedMode && privilege.permission && (
              <Box mt="3">
                <Text size="2">
                  <a
                    href={apiHubHref({ permission: privilege.permission })}
                    className="privilege-profile-page__api-hub-link"
                  >
                    {PrivilegesStrings.PROFILE.VIEW_API_ENDPOINTS_REQUIRING_PERMISSION}
                  </a>{' '}
                  <Text size="1" color="gray" as="span">
                    {PrivilegesStrings.PROFILE.OPENS_API_HUB(<Text as="span" style={{ fontFamily: 'var(--font-mono)' }}>?permission=</Text>)}
                  </Text>
                </Text>
              </Box>
            )}
          </Tabs.Content>

          <Tabs.Content value="roles">
            <Box className="privilege-profile-page__pane">
              <EntityTable<Role>
                data={rolesUsing}
                columns={roleColumns}
                getRowKey={(r) => r.id}
                loading={loading}
                loadingMessage={PrivilegesStrings.PROFILE.COMPUTING_ROLES}
                ariaLabel={PrivilegesStrings.PROFILE.ROLES_GRANT_PRIVILEGE_ARIA}
                emptyState={
                  <EmptyState
                    title={PrivilegesStrings.PROFILE.NO_ROLES_USE_PRIVILEGE}
                    description={PrivilegesStrings.PROFILE.NO_ROLES_CURRENTLY_GRANT_PRIVILEGE}
                    icon={Shield}
                  />
                }
              />
            </Box>
          </Tabs.Content>

          <Tabs.Content value="users">
            <Box className="privilege-profile-page__pane">
              <EntityTable<User>
                data={usersWithAccess}
                columns={userColumns}
                getRowKey={(u) => u.userId}
                loading={loading}
                loadingMessage={PrivilegesStrings.PROFILE.COMPUTING_USERS}
                ariaLabel={PrivilegesStrings.PROFILE.USERS_WHO_HAVE_PRIVILEGE_ARIA}
                emptyState={
                  <EmptyState
                    title={PrivilegesStrings.PROFILE.NO_USERS_HAVE_PRIVILEGE}
                    description={PrivilegesStrings.PROFILE.NO_USERS_HAVE_PRIVILEGE_THROUGH}
                    icon={Users}
                  />
                }
              />
            </Box>
          </Tabs.Content>
        </Box>
      </Tabs.Root>
    </Box>
  );
}

export default PrivilegeProfilePage;
