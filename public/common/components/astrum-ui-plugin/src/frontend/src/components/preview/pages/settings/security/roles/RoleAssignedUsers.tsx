

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { Box, Flex, Text, Callout } from '@radix-ui/themes';
import { Info, User as UserIcon } from 'lucide-react';
import { LoadingState, ErrorState, EntityTable, TableColumn } from '../../../../shared';
import { useUsersApi } from '../../security/users/useUsersApi';
import { User } from '../../security/users/types';
import { fetchAllUsersAcrossSources } from '../../system/api/hooks/securityDirectoryCache';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface RoleAssignedUsersProps {
  roleId: string;
}

/**
 * RoleAssignedUsers - Displays users who have the specified role assigned.
 */
export function RoleAssignedUsers({ roleId }: RoleAssignedUsersProps) {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { fetchUsers, fetchSources } = useUsersApi();

  const loadUsers = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // Fetch users from ALL sources (default, LDAP, SAML, OAuth2, etc.)
      // to ensure we show all users assigned to this role
      const allUsers = await fetchAllUsersAcrossSources(fetchSources, fetchUsers);
      const assignedUsers = allUsers.filter(user =>
        user.roles?.includes(roleId) || user.externalRoles?.includes(roleId)
      );
      setUsers(assignedUsers);
    } catch (err: any) {
      setError(err.message || RolesStrings.ASSIGNED_USERS.FAILED_LOAD_ASSIGNED_USERS);
    } finally {
      setLoading(false);
    }
  }, [roleId, fetchUsers, fetchSources]);

  useEffect(() => {
    loadUsers();
  }, [loadUsers]);

  const columns: TableColumn<User>[] = useMemo(() => [
    {
      id: 'userId',
      header: RolesStrings.ASSIGNED_USERS.USER_ID,
      accessor: (user) => (
        <Flex align="center" gap="2">
          <UserIcon size={16} color="var(--blue-9)" />
          <Text weight="medium">{user.userId}</Text>
        </Flex>
      ),
      sortable: true,
    },
    {
      id: 'name',
      header: CommonStrings.LABELS.NAME,
      accessor: (user) => `${user.firstName} ${user.lastName}`,
      sortable: true,
    },
    {
      id: 'email',
      header: RolesStrings.ASSIGNED_USERS.EMAIL,
      accessor: (user) => user.emailAddress || '—',
      sortable: true,
    },
    {
      id: 'source',
      header: RolesStrings.ASSIGNED_USERS.SOURCE,
      accessor: (user) => user.source,
      sortable: true,
    }
  ], []);

  if (loading) return <LoadingState message={RolesStrings.ASSIGNED_USERS.CHECKING_ASSIGNED_USERS} />;
  if (error) return <ErrorState message={error} onRetry={loadUsers} />;

  return (
    <Box p="4" className="role-assigned-users">
      <Callout.Root color="blue" mb="4" size="1">
        <Callout.Icon>
          <Info size={16} />
        </Callout.Icon>
        <Callout.Text>
          {RolesStrings.ASSIGNED_USERS.SHOWING_DIRECT_USERS(<strong>{roleId}</strong>)}
        </Callout.Text>
      </Callout.Root>

      {users.length > 0 ? (
        <EntityTable<User>
          data={users}
          columns={columns}
          getRowKey={(user) => user.userId}
          ariaLabel={RolesStrings.ASSIGNED_USERS.ASSIGNED_USERS_LIST_ARIA}
        />
      ) : (
        <Flex 
          direction="column" 
          align="center" 
          justify="center" 
          p="6" 
          style={{ background: 'var(--gray-2)', borderRadius: 'var(--radius-3)' }}
        >
          <Text color="gray" size="2">{RolesStrings.ASSIGNED_USERS.NO_USERS_CURRENTLY_ASSIGNED_ROLE}</Text>
        </Flex>
      )}
    </Box>
  );
}

export default RoleAssignedUsers;
