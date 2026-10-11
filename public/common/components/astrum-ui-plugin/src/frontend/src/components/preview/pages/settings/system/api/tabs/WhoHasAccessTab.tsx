

import React, { useMemo, useState } from 'react';
import { Box, Flex, Heading, Spinner, Table, Text, TextField } from '@radix-ui/themes';

import { getSourceLabel } from '../../../security/users/types';
import { useEndpointAccess } from '../hooks/useEndpointAccess';
import type { MergedApiEndpoint } from '../utils/mergeSwaggerPermissions';
import ApiStrings from '../../../../../constants/pages/settings/system/ApiStrings';
import plural from '../../../../../../../interface/plural';

const ROLES_PROFILE_BASE = 'preview/admin/security/roles';
const USERS_BASE = 'preview/admin/security/users';

function roleProfileHash(roleId: string): string {
  return `#${ROLES_PROFILE_BASE}/${encodeURIComponent(roleId)}/profile`;
}

function userProfileHash(userId: string, source: string): string {
  return `#${USERS_BASE}/${encodeURIComponent(userId)}/${encodeURIComponent(source)}/profile`;
}

export interface WhoHasAccessTabProps {
  row: MergedApiEndpoint;
  /** When false, skip loading security data (tab not visible) */
  active: boolean;
}

export function WhoHasAccessTab({ row, active }: WhoHasAccessTabProps) {
  const { loading, error, qualifyingRoles, usersWithAccess, noMappedPermissions } = useEndpointAccess(
    row,
    active
  );
  const [userFilter, setUserFilter] = useState('');

  const filteredUsers = useMemo(() => {
    const q = userFilter.trim().toLowerCase();
    if (!q) {
      return usersWithAccess;
    }
    return usersWithAccess.filter(({ user: u }) => {
      const blob = `${u.userId} ${u.firstName} ${u.lastName} ${u.emailAddress}`.toLowerCase();
      return blob.includes(q);
    });
  }, [usersWithAccess, userFilter]);

  const roleLabelById = useMemo(() => {
    const m = new Map<string, string>();
    for (const { role } of qualifyingRoles) {
      m.set(role.id, role.name);
    }
    return m;
  }, [qualifyingRoles]);

  const anonymous = row.permission && !row.permission.authenticated;

  return (
    <Box className="api-who-has-access-tab" data-testid="api-who-has-access-tab">
      {anonymous && (
        <Box mb="3" p="2" className="api-who-has-access-tab__banner">
          <Text size="2">
            {ApiStrings.WHO_HAS_ACCESS_TAB.ENDPOINT_ALLOWS_ANONYMOUS_ACCESS_WHEN}
          </Text>
        </Box>
      )}

      {noMappedPermissions && (
        <Text size="2" color="gray" mb="3" as="p">
          {ApiStrings.WHO_HAS_ACCESS_TAB.NO_PERMISSION_STRINGS_MAPPED_OPERATION}
        </Text>
      )}

      {error && (
        <Text size="2" color="red" mb="3">
          {error}
        </Text>
      )}

      {loading && (
        <Flex align="center" gap="2" py="4">
          <Spinner />
          <Text size="2">{ApiStrings.WHO_HAS_ACCESS_TAB.LOADING_ROLES_USERS}</Text>
        </Flex>
      )}

      {!((loading || error ) || noMappedPermissions ) && qualifyingRoles.length === 0 && (
        <Text size="2" color="gray" mb="3">
          {ApiStrings.WHO_HAS_ACCESS_TAB.NO_ROLES_CONTAIN_REQUIRED_PRIVILEGES}
        </Text>
      )}

      {!(loading || noMappedPermissions ) && qualifyingRoles.length > 0 && (
        <>
          <Heading as="h3" size="3" weight="medium" mb="2">
            {ApiStrings.WHO_HAS_ACCESS_TAB.ROLES}
          </Heading>
          <Text size="1" color="gray" mb="2">
            {plural(ApiStrings.WHO_HAS_ACCESS_TAB.ROLES_SATISFY, row.permission?.permissions?.length ?? 0)}{' '}
            ({row.permission?.permissions?.[0]?.logical === 'OR' ? ApiStrings.WHO_HAS_ACCESS_TAB.LOGICAL_ANY : ApiStrings.WHO_HAS_ACCESS_TAB.LOGICAL_ALL}).
          </Text>
          <Table.Root variant="surface" mb="4">
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>{ApiStrings.WHO_HAS_ACCESS_TAB.ROLE}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{ApiStrings.WHO_HAS_ACCESS_TAB.USERS}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{ApiStrings.WHO_HAS_ACCESS_TAB.NOTES}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {qualifyingRoles.map(({ role, userCount }) => (
                <Table.Row key={role.id}>
                  <Table.Cell>
                    <a href={roleProfileHash(role.id)} className="api-who-has-access-tab__link">
                      {role.name}
                    </a>
                    <Text size="1" color="gray" as="div">
                      {role.id}
                    </Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="2">{userCount}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    {role.readOnly && (
                      <Text size="1" color="gray">
                        {ApiStrings.WHO_HAS_ACCESS_TAB.READ_ONLY_ROLE_E_G}
                      </Text>
                    )}
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>

          <Flex align="center" justify="between" mb="2" wrap="wrap" gap="2">
            <Heading as="h3" size="3" weight="medium">
              {ApiStrings.WHO_HAS_ACCESS_TAB.USERS}
            </Heading>
            <Box style={{ minWidth: 200, flex: '1 1 200px' }}>
              <TextField.Root
                placeholder={ApiStrings.WHO_HAS_ACCESS_TAB.FILTER_USERS_PLACEHOLDER}
                value={userFilter}
                onChange={(e) => setUserFilter(e.target.value)}
                aria-label={ApiStrings.WHO_HAS_ACCESS_TAB.FILTER_USERS_ACCESS_ARIA}
              />
            </Box>
          </Flex>

          {filteredUsers.length === 0 ? (
            <Text size="2" color="gray">
              {ApiStrings.WHO_HAS_ACCESS_TAB.NO_USERS_CURRENTLY_HAVE_ACCESS}
            </Text>
          ) : (
            <Table.Root variant="surface">
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell>{ApiStrings.WHO_HAS_ACCESS_TAB.USER}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{ApiStrings.WHO_HAS_ACCESS_TAB.SOURCE}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{ApiStrings.WHO_HAS_ACCESS_TAB.GRANTING_ROLES}</Table.ColumnHeaderCell>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {filteredUsers.map(({ user, grantingRoleIds }) => (
                  <Table.Row key={`${user.userId}-${user.source}`}>
                    <Table.Cell>
                      <a href={userProfileHash(user.userId, user.source)} className="api-who-has-access-tab__link">
                        {user.userId}
                      </a>
                      <Text size="1" color="gray" as="div">
                        {user.firstName} {user.lastName}
                      </Text>
                    </Table.Cell>
                    <Table.Cell>
                      <Text size="2">{getSourceLabel(user.source)}</Text>
                      {user.readOnly && (
                        <Text size="1" color="gray" as="div">
                          {ApiStrings.WHO_HAS_ACCESS_TAB.EXTERNAL_READ_ONLY_ASTRUM}
                        </Text>
                      )}
                    </Table.Cell>
                    <Table.Cell>
                      <Text size="2">
                        {grantingRoleIds.map((rid, i) => (
                          <span key={rid}>
                            {i > 0 ? ', ' : ''}
                            <a href={roleProfileHash(rid)} className="api-who-has-access-tab__link">
                              {roleLabelById.get(rid) || rid}
                            </a>
                          </span>
                        ))}
                      </Text>
                    </Table.Cell>
                  </Table.Row>
                ))}
              </Table.Body>
            </Table.Root>
          )}
        </>
      )}
    </Box>
  );
}
