

import React from 'react';
import {
  Box,
  Flex,
  Text,
  Badge,
  Button,
  Card,
  Spinner,
  Table,
  Separator,
} from '@radix-ui/themes';
import {
  Key,
  Users as UsersIcon,
  User,
  Ghost,
  CheckCircle,
  XCircle,
} from 'lucide-react';
import ExtJS from '../../../../../../interface/ExtJS';
import { useRepositoryAccessSecurity } from './useRepositoryAccessSecurity';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

const USERS_DISPLAY_LIMIT = 10;

interface RepositoryAccessSecurityTabProps {
  repositoryName: string;
  repositoryFormat?: string;
}

/**
 * RepositoryAccessSecurityTab - Settings-page tab that surfaces
 * repo-scoped privileges, roles, users, and anonymous-access status.
 * Self-fetching; mounts only when the tab is opened (Radix Tabs unmounts
 * inactive content), so the security REST calls do not block Settings-page
 * first paint.
 *
 * Each section is gated on its own read permission so users can see just
 * the slice they have access to (mirrors the ratified Tasks & Capabilities
 * pattern). The corresponding fetch is skipped when the perm is missing.
 */
export function RepositoryAccessSecurityTab({
  repositoryName,
  repositoryFormat,
}: RepositoryAccessSecurityTabProps): JSX.Element {
  const canReadPrivileges = ExtJS.checkPermission('astrum:privileges:read');
  const canReadRoles = ExtJS.checkPermission('astrum:roles:read');
  const canReadUsers = ExtJS.checkPermission('astrum:users:read');
  const canReadAnonymous = ExtJS.checkPermission('astrum:settings:read');

  const { privileges, roles, users, anonymousAccess, loading, error, refetch } =
    useRepositoryAccessSecurity(repositoryName, {
      canReadPrivileges,
      canReadRoles,
      canReadUsers,
      canReadAnonymous,
      repositoryFormat,
    });

  if (loading) {
    return (
      <Flex justify="center" align="center" py="9" data-testid="access-security-loading">
        <Spinner size="3" />
      </Flex>
    );
  }

  if (error) {
    return (
      <Card size="2">
        <Flex direction="column" align="center" gap="3" py="6">
          <Text color="red" size="2">
            {RepositoriesPageStrings.ACCESS_SECURITY.FAILED_LOAD_ACCESS_SECURITY_DATA}
          </Text>
          <Text size="1" color="gray">
            {error}
          </Text>
          <Button variant="outline" size="2" onClick={refetch}>
            {CommonStrings.ACTIONS.RETRY}
          </Button>
        </Flex>
      </Card>
    );
  }

  const extraUsers = Math.max(0, users.length - USERS_DISPLAY_LIMIT);

  return (
    <Flex direction="column" gap="4">
      {canReadPrivileges && (
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" gap="2">
              <Key size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">
                {RepositoriesPageStrings.ACCESS_SECURITY.REPOSITORY_PRIVILEGES}
              </Text>
              <Badge variant="soft" color="gray" size="1">
                {plural(RepositoriesPageStrings.ACCESS_SECURITY.PRIVILEGE_COUNT, privileges.length)}
              </Badge>
            </Flex>

            <Separator size="4" />

            {privileges.length > 0 ? (
              <Flex direction="column" gap="2">
                {privileges.map((privilege) => (
                  <Flex
                    key={privilege.name}
                    align="center"
                    justify="between"
                    gap="3"
                  >
                    {/* The name and description wrap on their side; the actions badge stays at the right of every row */}
                    <Flex align="center" gap="2" wrap="wrap" minWidth="0">
                      <Text size="1" weight="medium" style={{ overflowWrap: 'anywhere' }}>
                        <code>{privilege.name}</code>
                      </Text>
                      {privilege.description && (
                        <Text size="1" color="gray">
                          ({privilege.description})
                        </Text>
                      )}
                    </Flex>
                    <Badge
                      size="1"
                      color={privilege.actions?.includes('*') ? 'green' : 'blue'}
                      style={{ flexShrink: 0 }}
                    >
                      {privilege.actions?.join(', ') || RepositoriesPageStrings.ACCESS_SECURITY.READ}
                    </Badge>
                  </Flex>
                ))}
              </Flex>
            ) : (
              <Box py="4">
                <Text size="2" color="gray">
                  {RepositoriesPageStrings.ACCESS_SECURITY.NO_PRIVILEGES_TARGET_REPOSITORY}
                </Text>
              </Box>
            )}
          </Flex>
        </Card>
      )}

      {canReadRoles && (
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" gap="2">
              <UsersIcon size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">
                {RepositoriesPageStrings.ACCESS_SECURITY.ROLES_ACCESS}
              </Text>
              <Badge variant="soft" color="gray" size="1">
                {plural(RepositoriesPageStrings.ACCESS_SECURITY.ROLE_COUNT, roles.length)}
              </Badge>
            </Flex>

            <Separator size="4" />

            {roles.length > 0 ? (
              <Table.Root variant="surface" size="1">
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeaderCell>{RepositoriesPageStrings.ACCESS_SECURITY.ROLE_2}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{RepositoriesPageStrings.ACCESS_SECURITY.PRIVILEGES}</Table.ColumnHeaderCell>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {roles.map((role) => (
                    <Table.Row key={role.id}>
                      <Table.Cell>
                        <Text size="1" weight="medium">
                          {role.name}
                        </Text>
                        {role.description && (
                          <Text size="1" color="gray" as="div">
                            {role.description}
                          </Text>
                        )}
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="1" color="gray">
                          {plural(RepositoriesPageStrings.ACCESS_SECURITY.PRIVILEGE_COUNT, role.privileges.length)}
                        </Text>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              </Table.Root>
            ) : (
              <Box py="4">
                <Text size="2" color="gray">
                  {RepositoriesPageStrings.ACCESS_SECURITY.NO_ROLES_REFERENCE_PRIVILEGES_REPOSITORY}
                </Text>
              </Box>
            )}
          </Flex>
        </Card>
      )}

      {canReadUsers && (
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" gap="2">
              <User size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">
                {RepositoriesPageStrings.ACCESS_SECURITY.USERS_ACCESS}
              </Text>
              <Badge variant="soft" color="gray" size="1">
                {plural(RepositoriesPageStrings.ACCESS_SECURITY.USER_COUNT, users.length)}
              </Badge>
            </Flex>

            <Separator size="4" />

            {users.length > 0 ? (
              <>
                <Table.Root variant="surface" size="1">
                  <Table.Header>
                    <Table.Row>
                      <Table.ColumnHeaderCell>{RepositoriesPageStrings.ACCESS_SECURITY.USER_2}</Table.ColumnHeaderCell>
                      <Table.ColumnHeaderCell>{RepositoriesPageStrings.ACCESS_SECURITY.VIA_ROLE}</Table.ColumnHeaderCell>
                    </Table.Row>
                  </Table.Header>
                  <Table.Body>
                    {users.slice(0, USERS_DISPLAY_LIMIT).map((user) => (
                      <Table.Row key={user.userId}>
                        <Table.Cell>
                          <Text size="1" weight="medium">
                            {user.userId}
                          </Text>
                          {(user.firstName || user.lastName) && (
                            <Text size="1" color="gray">
                              {' '}
                              ({[user.firstName, user.lastName].filter(Boolean).join(' ')})
                            </Text>
                          )}
                        </Table.Cell>
                        <Table.Cell>
                          <Text size="1">{user.roles?.join(', ') || '—'}</Text>
                        </Table.Cell>
                      </Table.Row>
                    ))}
                  </Table.Body>
                </Table.Root>
                {extraUsers > 0 && (
                  <Text size="1" color="gray">
                    {plural(RepositoriesPageStrings.ACCESS_SECURITY.MORE_USERS, extraUsers)}
                  </Text>
                )}
              </>
            ) : (
              <Box py="4">
                <Text size="2" color="gray">
                  {RepositoriesPageStrings.ACCESS_SECURITY.NO_USERS_HAVE_ACCESS_VIA}
                </Text>
              </Box>
            )}
          </Flex>
        </Card>
      )}

      {canReadAnonymous && (
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" gap="2">
              <Ghost size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">
                {RepositoriesPageStrings.ACCESS_SECURITY.ANONYMOUS_ACCESS}
              </Text>
              {anonymousAccess && (
                <Badge
                  color={anonymousAccess.enabled ? 'green' : 'gray'}
                  size="1"
                >
                  {anonymousAccess.enabled ? (
                    <>
                      <CheckCircle size={12} /> {RepositoriesPageStrings.ACCESS_SECURITY.ENABLED}
                    </>
                  ) : (
                    <>
                      <XCircle size={12} /> {RepositoriesPageStrings.ACCESS_SECURITY.DISABLED}
                    </>
                  )}
                </Badge>
              )}
            </Flex>

            <Separator size="4" />

            {anonymousAccess ? (
              <Flex direction="column" gap="2">
                {anonymousAccess.enabled && (
                  <>
                    <Flex align="center" justify="between">
                      <Text size="1" color="gray">
                        {RepositoriesPageStrings.ACCESS_SECURITY.USER_ID}
                      </Text>
                      <Text size="1">{anonymousAccess.userId || '—'}</Text>
                    </Flex>
                    <Flex align="center" justify="between">
                      <Text size="1" color="gray">
                        {RepositoriesPageStrings.ACCESS_SECURITY.REALM}
                      </Text>
                      <Text size="1">
                        {anonymousAccess.realmName || anonymousAccess.realm || '—'}
                      </Text>
                    </Flex>
                  </>
                )}
                {!anonymousAccess.enabled && (
                  <Text size="2" color="gray">
                    {RepositoriesPageStrings.ACCESS_SECURITY.ANONYMOUS_USERS_CANNOT_ACCESS_REPOSITORY}
                  </Text>
                )}
              </Flex>
            ) : (
              <Box py="4">
                <Text size="2" color="gray">
                  {RepositoriesPageStrings.ACCESS_SECURITY.ANONYMOUS_ACCESS_SETTINGS_NOT_AVAILABLE}
                </Text>
              </Box>
            )}
          </Flex>
        </Card>
      )}
    </Flex>
  );
}

export default RepositoryAccessSecurityTab;
