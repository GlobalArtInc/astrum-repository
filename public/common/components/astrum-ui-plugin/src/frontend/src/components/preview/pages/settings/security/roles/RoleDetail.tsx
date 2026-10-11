

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, AlertTriangle } from 'lucide-react';

import {
  SettingsFormSection,
  SettingsAlert,
} from '../../../../shared/form';
import { MetadataGrid } from '../../../../shared';
import { RoleForm } from './RoleForm';
import { useRolesApi } from './useRolesApi';
import {
  RoleDetailProps,
  isReadOnlyRole,
  formatRoleSourceDisplay,
} from './types';

import './RoleDetail.scss';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * RoleDetail - Detailed view and edit form for a single role
 */
export function RoleDetail({
  role,
  loading,
  canEdit,
  canDelete,
  onDelete,
  onCancel,
  onComplete,
  error,
}: RoleDetailProps) {
  const { loading: apiLoading, error: apiError, setError } = useRolesApi();

  const isReadOnly = role ? isReadOnlyRole(role) : false;
  const showDeleteButton = canDelete && role && !isReadOnly;

  // Loading state
  if (loading) {
    return (
      <Flex align="center" justify="center" className="role-detail__loading">
        <Loader2 size={24} className="role-detail__spinner" />
        <Text size="2">{RolesStrings.DETAIL.LOADING_ROLE_DETAILS}</Text>
      </Flex>
    );
  }

  // No role found
  if (!role) {
    return (
      <Box className="role-detail__not-found">
        <AlertTriangle size={24} />
        <Text size="2">{RolesStrings.DETAIL.ROLE_NOT_FOUND}</Text>
      </Box>
    );
  }

  // Read-only view for users without edit permission or for read-only roles
  if (!canEdit || isReadOnly) {
    return (
      <Box className="role-detail">
        {/* Error Messages */}
        {apiError && (
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {apiError}
          </SettingsAlert>
        )}

        {isReadOnly && (
          <Box className="role-detail__readonly-banner">
            <SettingsAlert type="info">
              {RolesStrings.DETAIL.ROLE_READ_ONLY_CANNOT_MODIFIED}
            </SettingsAlert>
          </Box>
        )}

        <SettingsFormSection title={RolesStrings.DETAIL.ROLE_INFORMATION} defaultOpen>
          <MetadataGrid
            items={[
              { label: 'ID', value: role.id },
              { label: CommonStrings.LABELS.NAME, value: role.name },
              { label: CommonStrings.LABELS.DESCRIPTION, value: role.description || '—' },
              { label: RolesStrings.DETAIL.SOURCE, value: formatRoleSourceDisplay(role.source) },
            ]}
          />
        </SettingsFormSection>

        <SettingsFormSection title={RolesStrings.DETAIL.PRIVILEGES}>
          <Flex direction="column" gap="2">
            {role.privileges && role.privileges.length > 0 ? (
              role.privileges.map((priv) => (
                <Text
                  key={priv}
                  size="2"
                  p="2"
                  style={{
                    background: 'var(--gray-3)',
                    borderRadius: 'var(--radius-2)',
                    fontFamily: 'var(--font-mono)',
                  }}
                >
                  {priv}
                </Text>
              ))
            ) : (
              <Text size="2" color="gray" style={{ fontStyle: 'italic' }}>
                {RolesStrings.DETAIL.NO_PRIVILEGES_ASSIGNED}
              </Text>
            )}
          </Flex>
        </SettingsFormSection>

        <SettingsFormSection title={RolesStrings.DETAIL.CONTAINED_ROLES}>
          <Flex direction="column" gap="2">
            {role.roles && role.roles.length > 0 ? (
              role.roles.map((r) => (
                <Text
                  key={r}
                  size="2"
                  p="2"
                  style={{
                    background: 'var(--gray-3)',
                    borderRadius: 'var(--radius-2)',
                    fontFamily: 'var(--font-mono)',
                  }}
                >
                  {r}
                </Text>
              ))
            ) : (
              <Text size="2" color="gray" style={{ fontStyle: 'italic' }}>
                {RolesStrings.DETAIL.NO_CONTAINED_ROLES}
              </Text>
            )}
          </Flex>
        </SettingsFormSection>

      </Box>
    );
  }

  // Edit view
  return (
    <Box className="role-detail">
      {/* Error Messages */}
      {apiError && (
        <SettingsAlert type="error" onClose={() => setError(null)}>
          {apiError}
        </SettingsAlert>
      )}

      {/* Role Form */}
      <RoleForm
        role={role}
        isCreate={false}
        onCancel={onCancel}
        onDelete={showDeleteButton ? onDelete : undefined}
        onComplete={onComplete}
        loading={loading || apiLoading}
        error={error}
      />
    </Box>
  );
}

export default RoleDetail;
