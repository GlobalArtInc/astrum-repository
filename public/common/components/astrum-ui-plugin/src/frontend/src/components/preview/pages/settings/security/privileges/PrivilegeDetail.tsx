

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
import PrivilegesStrings from '../../../../constants/pages/settings/security/PrivilegesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

// Names of the properties of a privilege, which the server sends by their ids
const PROPERTY_LABELS: Record<string, () => string> = {
  pattern: () => PrivilegesStrings.FORM_MACHINE.PATTERN,
  domain: () => PrivilegesStrings.FORM_MACHINE.DOMAIN,
  actions: () => CommonStrings.LABELS.ACTIONS,
  format: () => PrivilegesStrings.FORM_MACHINE.REPOSITORY_FORMAT,
  repository: () => PrivilegesStrings.FORM_MACHINE.REPOSITORY,
  contentSelector: () => PrivilegesStrings.FORM_MACHINE.CONTENT_SELECTOR,
  name: () => PrivilegesStrings.FORM_MACHINE.SCRIPT_NAME,
};

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
        <Text size="2">{PrivilegesStrings.DETAIL.LOADING_PRIVILEGE_DETAILS}</Text>
      </Flex>
    );
  }

  // No privilege found
  if (!privilege) {
    return (
      <Box className="privilege-detail__not-found">
        <AlertTriangle size={24} />
        <Text size="2">{PrivilegesStrings.DETAIL.PRIVILEGE_NOT_FOUND}</Text>
      </Box>
    );
  }

  // Read-only view for users without edit permission or for read-only privileges
  if (!canEdit || isReadOnly) {
    // The pattern of a wildcard privilege is its permission, which the section above already shows
    const properties = Object.entries(privilege.properties ?? {}).filter(
      ([key, value]) => !(key === 'pattern' && value === privilege.permission)
    );
    return (
      <Box className="privilege-detail">
        {isReadOnly && (
          <Box className="privilege-detail__readonly-banner">
            <SettingsAlert type="info">
              {PrivilegesStrings.DETAIL.PRIVILEGE_READ_ONLY_CANNOT_MODIFIED}
            </SettingsAlert>
          </Box>
        )}

        <SettingsFormSection title={PrivilegesStrings.DETAIL.PRIVILEGE_INFORMATION} defaultOpen>
          <Box className="privilege-detail__info">
            <Flex className="privilege-detail__row">
              <Text size="2" weight="medium" className="privilege-detail__label">{CommonStrings.LABELS.NAME}</Text>
              <Text size="2">{privilege.name}</Text>
            </Flex>
            <Flex className="privilege-detail__row">
              <Text size="2" weight="medium" className="privilege-detail__label">{CommonStrings.LABELS.DESCRIPTION}</Text>
              <Text size="2">{privilege.description || '-'}</Text>
            </Flex>
            <Flex className="privilege-detail__row">
              <Text size="2" weight="medium" className="privilege-detail__label">{PrivilegesStrings.DETAIL.PERMISSION}</Text>
              <code className="privilege-detail__permission">{privilege.permission}</code>
            </Flex>
          </Box>
        </SettingsFormSection>

        {properties.length > 0 && (
          <SettingsFormSection title={PrivilegesStrings.DETAIL.PROPERTIES}>
            <Box className="privilege-detail__properties">
              {properties.map(([key, value]) => (
                <Flex key={key} className="privilege-detail__row">
                  <Text size="2" weight="medium" className="privilege-detail__label">
                    {PROPERTY_LABELS[key]?.() ?? key}
                  </Text>
                  <Text size="2">{value || '-'}</Text>
                </Flex>
              ))}
            </Box>
          </SettingsFormSection>
        )}

        <Flex gap="3" className="privilege-detail__actions">
          <SettingsButton variant="secondary" onClick={onCancel}>
            {CommonStrings.ACTIONS.BACK_TO_LIST}
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


