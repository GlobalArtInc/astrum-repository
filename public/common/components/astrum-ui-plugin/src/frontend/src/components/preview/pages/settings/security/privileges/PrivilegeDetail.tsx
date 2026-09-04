

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, AlertTriangle } from 'lucide-react';

import {
  SettingsFormSection,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import { PrivilegeForm } from './PrivilegeForm';
import { usePrivilegesApi } from './usePrivilegesApi';
import {
  PrivilegeDetailProps,
  isReadOnlyPrivilege,
} from './types';

import './PrivilegeDetail.scss';

/**
 * PrivilegeDetail - Detailed view and edit form for a single privilege
 */
export function PrivilegeDetail({
  privilege,
  loading,
  canEdit,
  canDelete,
  onSave,
  onDelete,
  onCancel,
  error,
}: PrivilegeDetailProps) {
  const { loading: apiLoading, error: apiError, setError } = usePrivilegesApi();

  const isReadOnly = privilege ? isReadOnlyPrivilege(privilege) : false;
  const showDeleteButton = canDelete && privilege && !isReadOnly;

  // Loading state
  if (loading) {
    return (
      <Flex align="center" justify="center" className="privilege-detail__loading">
        <Loader2 size={24} className="privilege-detail__spinner" />
        <Text size="2">Loading privilege details...</Text>
      </Flex>
    );
  }

  // No privilege found
  if (!privilege) {
    return (
      <Box className="privilege-detail__not-found">
        <AlertTriangle size={24} />
        <Text size="2">Privilege not found</Text>
      </Box>
    );
  }

  // Read-only view for users without edit permission or for read-only privileges
  if (!canEdit || isReadOnly) {
    return (
      <Box className="privilege-detail">
        {isReadOnly && (
          <Box className="privilege-detail__readonly-banner">
            <SettingsAlert type="info">
              This privilege is read-only and cannot be modified.
            </SettingsAlert>
          </Box>
        )}

        <SettingsFormSection title="Privilege Information" defaultOpen>
          <Box className="privilege-detail__info">
            <Flex className="privilege-detail__row">
              <Text size="2" weight="medium" className="privilege-detail__label">Name</Text>
              <Text size="2">{privilege.name}</Text>
            </Flex>
            <Flex className="privilege-detail__row">
              <Text size="2" weight="medium" className="privilege-detail__label">Description</Text>
              <Text size="2">{privilege.description || '-'}</Text>
            </Flex>
            <Flex className="privilege-detail__row">
              <Text size="2" weight="medium" className="privilege-detail__label">Permission</Text>
              <code className="privilege-detail__permission">{privilege.permission}</code>
            </Flex>
          </Box>
        </SettingsFormSection>

        {privilege.properties && Object.keys(privilege.properties).length > 0 && (
          <SettingsFormSection title="Properties">
            <Box className="privilege-detail__properties">
              {Object.entries(privilege.properties).map(([key, value]) => (
                <Flex key={key} className="privilege-detail__row">
                  <Text size="2" weight="medium" className="privilege-detail__label">
                    {key}
                  </Text>
                  <Text size="2">{value || '-'}</Text>
                </Flex>
              ))}
            </Box>
          </SettingsFormSection>
        )}

        <Flex gap="3" className="privilege-detail__actions">
          <SettingsButton variant="secondary" onClick={onCancel}>
            Back to List
          </SettingsButton>
        </Flex>
      </Box>
    );
  }

  // Edit view
  return (
    <Box className="privilege-detail">
      {/* Error Messages */}
      {apiError && (
        <SettingsAlert type="error" onClose={() => setError(null)}>
          {apiError}
        </SettingsAlert>
      )}

      {/* Privilege Form */}
      <PrivilegeForm
        privilege={privilege}
        isCreate={false}
        typeId={privilege.type}
        onSave={onSave}
        onCancel={onCancel}
        onDelete={showDeleteButton ? onDelete : undefined}
        loading={loading || apiLoading}
        error={error}
      />
    </Box>
  );
}

export default PrivilegeDetail;


