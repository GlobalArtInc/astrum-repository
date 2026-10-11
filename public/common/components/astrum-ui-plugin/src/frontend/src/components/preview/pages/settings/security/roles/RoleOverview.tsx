

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { SettingsFormSection } from '../../../../shared/form';
import { Role, formatRoleSourceDisplay } from './types';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface RoleOverviewProps {
  role: Role;
}

/**
 * RoleOverview - Displays basic metadata for a role in the profile view.
 */
export function RoleOverview({ role }: RoleOverviewProps) {
  return (
    <Box p="4" className="role-overview">
      <SettingsFormSection title={RolesStrings.OVERVIEW.ROLE_INFORMATION} defaultOpen>
        <Box className="role-detail__info">
          <Flex className="role-detail__row" mb="3">
            <Text size="2" weight="medium" style={{ width: '120px', color: 'var(--gray-11)' }}>{RolesStrings.OVERVIEW.ID}</Text>
            <Text size="2">{role.id}</Text>
          </Flex>
          <Flex className="role-detail__row" mb="3">
            <Text size="2" weight="medium" style={{ width: '120px', color: 'var(--gray-11)' }}>{CommonStrings.LABELS.NAME}</Text>
            <Text size="2">{role.name}</Text>
          </Flex>
          <Flex className="role-detail__row" mb="3">
            <Text size="2" weight="medium" style={{ width: '120px', color: 'var(--gray-11)' }}>{CommonStrings.LABELS.DESCRIPTION}</Text>
            <Text size="2">{role.description || '—'}</Text>
          </Flex>
          <Flex className="role-detail__row" mb="3">
            <Text size="2" weight="medium" style={{ width: '120px', color: 'var(--gray-11)' }}>{RolesStrings.OVERVIEW.SOURCE}</Text>
            <Text size="2">{formatRoleSourceDisplay(role.source)}</Text>
          </Flex>
        </Box>
      </SettingsFormSection>

      <SettingsFormSection title={RolesStrings.OVERVIEW.DIRECT_ASSIGNMENTS}>
        <Flex direction="column" gap="4">
          <Box>
            <Text size="2" weight="medium" mb="2" display="block">{RolesStrings.OVERVIEW.DIRECT_PRIVILEGES(role.privileges?.length || 0)}</Text>
            <Flex direction="column" gap="1">
              {role.privileges && role.privileges.length > 0 ? (
                role.privileges.map((priv) => (
                  <Text key={priv} size="2" p="2" style={{ background: 'var(--gray-3)', borderRadius: 'var(--radius-2)', fontFamily: 'var(--font-mono)' }}>
                    {priv}
                  </Text>
                ))
              ) : (
                <Text size="2" color="gray" style={{ fontStyle: 'italic' }}>{RolesStrings.OVERVIEW.NO_DIRECT_PRIVILEGES_ASSIGNED}</Text>
              )}
            </Flex>
          </Box>

          <Box>
            <Text size="2" weight="medium" mb="2" display="block">{RolesStrings.OVERVIEW.CONTAINED_ROLES(role.roles?.length || 0)}</Text>
            <Flex direction="column" gap="1">
              {role.roles && role.roles.length > 0 ? (
                role.roles.map((r) => (
                  <Text key={r} size="2" p="2" style={{ background: 'var(--gray-3)', borderRadius: 'var(--radius-2)', fontFamily: 'var(--font-mono)' }}>
                    {r}
                  </Text>
                ))
              ) : (
                <Text size="2" color="gray" style={{ fontStyle: 'italic' }}>{RolesStrings.OVERVIEW.NO_CONTAINED_ROLES}</Text>
              )}
            </Flex>
          </Box>
        </Flex>
      </SettingsFormSection>
    </Box>
  );
}

export default RoleOverview;
