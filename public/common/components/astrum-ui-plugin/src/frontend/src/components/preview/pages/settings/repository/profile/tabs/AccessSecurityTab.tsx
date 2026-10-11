

import React from 'react';
import { Box, Flex, Text, Badge, Table } from '@radix-ui/themes';
import {
  Key,
  Users,
  User,
  Ghost,
  CheckCircle,
  XCircle,
  Loader2,
} from 'lucide-react';
import { ClassicSettingsLink } from './classicSettingsLink';
import type {
  RepositoryProfileData,
  PrivilegeInfo,
  RoleInfo,
  UserWithAccess,
  AnonymousAccess,
} from '../types';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';

// =============================================================================
// Types
// =============================================================================

interface AccessSecurityTabProps {
  repository: RepositoryProfileData;
  privileges: PrivilegeInfo[];
  roles: RoleInfo[];
  users: UserWithAccess[];
  anonymousAccess: AnonymousAccess | null;
  loading?: boolean;
}

// =============================================================================
// Helper Components
// =============================================================================

interface ProfileSectionProps {
  title: string;
  icon: React.ElementType;
  editPath?: string;
  children: React.ReactNode;
}

function ProfileSection({ title, icon: Icon, editPath, children }: ProfileSectionProps): JSX.Element {
  return (
    <Box className="profile-section__card" mb="4">
      <Flex align="center" justify="between" mb="4" className="profile-section__header">
        <Flex align="center" gap="2">
          <Icon size={18} />
          <Text weight="bold">{title}</Text>
        </Flex>
        {editPath && (
          <ClassicSettingsLink previewPath={editPath} label={RepositoryProfileStrings.SECURITY_TAB.VIEW} />
        )}
      </Flex>
      {children}
    </Box>
  );
}

// =============================================================================
// Component
// =============================================================================

/**
 * AccessSecurityTab - Shows who has access to this repository
 *
 * Maps to Settings → Security menu:
 * - Privileges: Repository-specific privileges
 * - Roles with Access: Roles that include these privileges
 * - Users with Access: Users with these roles
 * - Anonymous Access: Can anonymous read/browse this repo
 */
export function AccessSecurityTab({
  repository,
  privileges,
  roles,
  users,
  anonymousAccess,
  loading,
}: AccessSecurityTabProps): JSX.Element {
  if (loading) {
    return (
      <Box className="profile-empty-state">
        <Loader2 size={48} className="profile-empty-state__icon profile-empty-state__icon--spinning" />
        <Text className="profile-empty-state__title">{RepositoryProfileStrings.SECURITY_TAB.LOADING_ACCESS_DATA}</Text>
        <Text className="profile-empty-state__message">
          {RepositoryProfileStrings.SECURITY_TAB.FETCHING_PRIVILEGES_ROLES_USERS_ACCESS}
        </Text>
      </Box>
    );
  }

  return (
    <Box>
      {/* Privileges Section */}
      <ProfileSection
        title={RepositoryProfileStrings.SECURITY_TAB.PRIVILEGES}
        icon={Key}
        editPath="preview/admin/security/privileges"
      >
        {privileges.length > 0 ? (
          <Flex direction="column" gap="2">
            {privileges.map((privilege) => (
              <Box key={privilege.name} className="profile-section__row profile-section__row--privilege">
                <Flex align="center" gap="2">
                  <code className="profile-section__value--code">{privilege.name}</code>
                  {privilege.description && (
                    <Text size="1" color="gray">({privilege.description})</Text>
                  )}
                </Flex>
                <Badge size="1" color={privilege.actions?.includes('*') ? 'green' : 'blue'}>
                  {privilege.actions?.join(', ') || RepositoryProfileStrings.SECURITY_TAB.READ}
                </Badge>
              </Box>
            ))}
          </Flex>
        ) : (
          <Text color="gray" size="2">
            {RepositoryProfileStrings.SECURITY_TAB.NO_SPECIFIC_PRIVILEGES_FOUND_REPOSITORY}
          </Text>
        )}
      </ProfileSection>

      {/* Roles with Access */}
      <ProfileSection
        title={RepositoryProfileStrings.SECURITY_TAB.ROLES_ACCESS}
        icon={Users}
        editPath="preview/admin/security/roles"
      >
        {roles.length > 0 ? (
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SECURITY_TAB.ROLE}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SECURITY_TAB.PRIVILEGES}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SECURITY_TAB.USERS}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {roles.map((role) => (
                <Table.Row key={role.id}>
                  <Table.Cell>
                    <Text weight="medium">{role.name}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="1" color="gray">
                      {RepositoryProfileStrings.SECURITY_TAB.PRIVILEGES_2(role.privilegeCount ?? 0)}
                    </Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Badge size="1">{role.userCount ?? 0}</Badge>
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        ) : (
          <Text color="gray" size="2">
            {RepositoryProfileStrings.SECURITY_TAB.NO_ROLES_FOUND_EXPLICIT_ACCESS}
          </Text>
        )}
      </ProfileSection>

      {/* Users with Access */}
      <ProfileSection
        title={RepositoryProfileStrings.SECURITY_TAB.USERS_ACCESS}
        icon={User}
        editPath="preview/admin/security/users"
      >
        {users.length > 0 ? (
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SECURITY_TAB.USER}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SECURITY_TAB.VIA_ROLE}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SECURITY_TAB.PERMISSIONS}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {users.slice(0, 10).map((user) => (
                <Table.Row key={user.userId}>
                  <Table.Cell>
                    <Text weight="medium">{user.userId}</Text>
                    {user.firstName && (
                      <Text size="1" color="gray"> ({user.firstName} {user.lastName})</Text>
                    )}
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="1">{user.roles?.join(', ') || '—'}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Badge size="1" color={user.permissions?.includes('Full Access') ? 'green' : 'blue'}>
                      {user.permissions?.join(', ') || RepositoryProfileStrings.SECURITY_TAB.READ}
                    </Badge>
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        ) : (
          <Text color="gray" size="2">
            {RepositoryProfileStrings.SECURITY_TAB.NO_USERS_FOUND_EXPLICIT_ACCESS}
          </Text>
        )}
        {users.length > 10 && (
          <Text size="1" color="gray" mt="2">
            {RepositoryProfileStrings.SECURITY_TAB.MORE_USERS(users.length - 10)}
          </Text>
        )}
      </ProfileSection>

      {/* Anonymous Access */}
      <ProfileSection
        title={RepositoryProfileStrings.SECURITY_TAB.ANONYMOUS_ACCESS}
        icon={Ghost}
        editPath="preview/admin/security/anonymous"
      >
        {anonymousAccess ? (
          <Flex direction="column" gap="2">
            <Box className="profile-section__row">
              <Text className="profile-section__label">{CommonStrings.LABELS.STATUS}</Text>
              <Flex align="center" gap="1">
                {anonymousAccess.enabled ? (
                  <>
                    <CheckCircle size={14} color="var(--green-11)" />
                    <Text className="profile-section__value">{RepositoryProfileStrings.SECURITY_TAB.ENABLED}</Text>
                  </>
                ) : (
                  <>
                    <XCircle size={14} color="var(--red-11)" />
                    <Text className="profile-section__value">{RepositoryProfileStrings.SECURITY_TAB.DISABLED}</Text>
                  </>
                )}
              </Flex>
            </Box>
            {anonymousAccess.enabled && (
              <>
                <Box className="profile-section__row">
                  <Text className="profile-section__label">{RepositoryProfileStrings.SECURITY_TAB.CAN_READ}</Text>
                  <Text className="profile-section__value">
                    {anonymousAccess.canRead ? RepositoryProfileStrings.SECURITY_TAB.YES : RepositoryProfileStrings.SECURITY_TAB.NO}
                  </Text>
                </Box>
                <Box className="profile-section__row">
                  <Text className="profile-section__label">{RepositoryProfileStrings.SECURITY_TAB.CAN_BROWSE}</Text>
                  <Text className="profile-section__value">
                    {anonymousAccess.canBrowse ? RepositoryProfileStrings.SECURITY_TAB.YES : RepositoryProfileStrings.SECURITY_TAB.NO}
                  </Text>
                </Box>
                <Box className="profile-section__row">
                  <Text className="profile-section__label">{RepositoryProfileStrings.SECURITY_TAB.REALM}</Text>
                  <Text className="profile-section__value">{anonymousAccess.realm || '—'}</Text>
                </Box>
              </>
            )}
          </Flex>
        ) : (
          <Text color="gray" size="2">
            {RepositoryProfileStrings.SECURITY_TAB.ANONYMOUS_ACCESS_SETTINGS_NOT_AVAILABLE}
          </Text>
        )}
      </ProfileSection>
    </Box>
  );
}

export default AccessSecurityTab;


