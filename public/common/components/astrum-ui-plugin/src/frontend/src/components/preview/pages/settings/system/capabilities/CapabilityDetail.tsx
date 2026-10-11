

import React, { useState, useEffect, useCallback } from 'react';
import { Box, Flex, Text, Badge, Tabs, Spinner } from '@radix-ui/themes';
import {
  AlertCircle,
  Power,
  PowerOff,
  Trash2,
  Info,
  Settings,
  FileText,
  Edit3,
} from 'lucide-react';
import DOMPurify from 'dompurify';
import { SettingsButton, SettingsAlert, SettingsFormSection, } from '../../../../shared/form';
import { ConfirmDialog, PageHeader, type BreadcrumbItem } from '../../../../shared';
import { ExtJS } from '../../../../../../interface/ExtJS';
import Permissions from '../../../../../../constants/Permissions';
import {
  Capability,
  CapabilityType,
  CapabilityFormData,
  getStateBadgeColor,
  getStateName,
} from './types';
import { DynamicCapabilityForm } from './DynamicCapabilityForm';
import { useCapabilitiesApi } from './useCapabilitiesApi';

import './CapabilityDetail.scss';
import CapabilitiesStrings from '../../../../constants/pages/settings/system/CapabilitiesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface CapabilityDetailProps {
  capability: Capability;
  onSave: (data: CapabilityFormData) => Promise<void>;
  onDelete: () => void;
  onEnable: () => void;
  onDisable: () => void;
  onBack: () => void;
  /** Breadcrumbs of the page header, which this view renders with the capability's name, state and actions */
  breadcrumbs?: BreadcrumbItem[];
  loading?: boolean;
  error?: string;
}

export function CapabilityDetail({
  capability,
  onSave,
  onDelete,
  onEnable,
  onDisable,
  onBack,
  breadcrumbs,
  loading = false,
  error,
}: CapabilityDetailProps) {
  const { fetchCapabilityTypes } = useCapabilitiesApi();
  const [capabilityType, setCapabilityType] = useState<CapabilityType | null>(null);
  const [loadingType, setLoadingType] = useState(true);
  const [activeTab, setActiveTab] = useState('summary');
  const [confirmAction, setConfirmAction] = useState<'delete' | 'disable' | null>(null);
  // Track enabled state locally for UI synchronization between header buttons and form checkbox
  const [isEnabled, setIsEnabled] = useState(capability.enabled);

  // Provider-independent, reactive permission checks (ASTRUM-54212). ExtJS.checkPermission alone
  // evaluates once at render and would not update if permissions load asynchronously after
  // mount, leaving the Enable/Disable and Delete actions hidden for a permitted non-admin until
  // refresh; ExtJS.usePermission with a hasUser dependency re-evaluates once the user loads.
  const hasUser = ExtJS.useUser() ?? false;
  const canUpdate = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.CAPABILITIES.UPDATE),
    [hasUser],
  );
  const canDelete = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.CAPABILITIES.DELETE),
    [hasUser],
  );

  useEffect(() => {
    const loadType = async () => {
      setLoadingType(true);
      try {
        const types = await fetchCapabilityTypes();
        const type = types.find((t) => t.id === capability.typeId);
        setCapabilityType(type || null);
      } catch (err) {
        console.error('Failed to load capability type:', err);
      } finally {
        setLoadingType(false);
      }
    };

    loadType();
  }, [capability.typeId, fetchCapabilityTypes]);

  const handleDelete = useCallback(() => {
    setConfirmAction('delete');
  }, []);

  const handleEnable = useCallback(async () => {
    setIsEnabled(true);
    try {
      await onEnable();
    } catch {
      setIsEnabled(false);
    }
  }, [onEnable]);

  const handleDisable = useCallback(async () => {
    if (capability.disableWarningMessage) {
      setConfirmAction('disable');
    } else {
      setIsEnabled(false);
      try {
        await onDisable();
      } catch {
        setIsEnabled(true);
      }
    }
  }, [capability.disableWarningMessage, onDisable]);

  const handleDisableConfirm = useCallback(async () => {
    setConfirmAction(null);
    setIsEnabled(false);
    try {
      await onDisable();
    } catch {
      setIsEnabled(true);
    }
  }, [onDisable]);

  const handleEnabledChange = useCallback((enabled: boolean) => {
    setIsEnabled(enabled);
  }, []);

  const actions = (
    <>
      {canUpdate && !isEnabled && (
        <SettingsButton
          variant="secondary"
          onClick={handleEnable}
          disabled={loading}
          icon={Power}
          data-analytics-id="as-capability-toggle-enable"
        >
          {CapabilitiesStrings.DETAIL.ENABLE}
        </SettingsButton>
      )}
      {canUpdate && isEnabled && (
        <SettingsButton
          variant="secondary"
          onClick={handleDisable}
          disabled={loading}
          icon={PowerOff}
          data-analytics-id="as-capability-toggle-disable"
        >
          {CapabilitiesStrings.DETAIL.DISABLE}
        </SettingsButton>
      )}
      {!capability.isSystem && (
        <SettingsButton
          variant="danger"
          onClick={handleDelete}
          disabled={!canDelete || loading}
          icon={Trash2}
          testId="capability-delete-button"
          data-analytics-id="as-capability-delete"
        >
          {CommonStrings.ACTIONS.DELETE}
        </SettingsButton>
      )}
    </>
  );

  return (
    <Box className="capability-detail" data-testid="capability-detail">
      <PageHeader
        title={capability.typeName}
        titleSuffix={
          <Badge color={getStateBadgeColor(capability.state)} variant="soft" data-testid="capability-state-badge">
            {getStateName(capability.state)}
          </Badge>
        }
        breadcrumbs={breadcrumbs}
        actions={actions}
        className="capability-detail__page-header"
      />

      {/* Error display at top level */}
      {error && (
        <Box className="capability-detail__error-banner">
          <SettingsAlert type="error">{error}</SettingsAlert>
        </Box>
      )}

      {/* Tabs */}
      <Tabs.Root value={activeTab} onValueChange={setActiveTab} className="capability-detail__tabs">
        <Tabs.List>
          <Tabs.Trigger value="summary">
            <Info size={16} />
            {CapabilitiesStrings.DETAIL.SUMMARY}
          </Tabs.Trigger>
          <Tabs.Trigger value="settings">
            <Settings size={16} />
            {CommonStrings.LABELS.SETTINGS}
          </Tabs.Trigger>
          <Tabs.Trigger value="status">
            <FileText size={16} />
            {CommonStrings.LABELS.STATUS}
          </Tabs.Trigger>
          <Tabs.Trigger value="about">
            <Info size={16} />
            {CapabilitiesStrings.DETAIL.ABOUT}
          </Tabs.Trigger>
        </Tabs.List>

        <Box className="capability-detail__tab-content">
          {/* Summary Tab */}
          <Tabs.Content value="summary">

            <Box className="capability-detail__summary-grid">
              <SummaryItem
                icon={<FileText size={16} />}
                label={CommonStrings.LABELS.DESCRIPTION}
                value={capability.description || '-'}
              />
              {capability.notes && (
                <SummaryItem
                  icon={<Edit3 size={16} />}
                  label={CapabilitiesStrings.DETAIL.NOTES}
                  value={capability.notes}
                />
              )}
              {/* Dynamic Tags */}
              {capability.tags &&
                Object.entries(capability.tags).map(([key, value]) => (
                  <SummaryItem
                    key={key}
                    label={key}
                    value={value}
                  />
                ))}
            </Box>
          </Tabs.Content>

          {/* Settings Tab */}
          <Tabs.Content value="settings">
            {loadingType ? (
              <Flex align="center" justify="center" className="capability-detail__loading">
                <Spinner size="3" />
                <Text size="2">{CapabilitiesStrings.DETAIL.LOADING_SETTINGS}</Text>
              </Flex>
            ) : !capabilityType ? (
              <Box className="capability-detail__error">
                <AlertCircle size={20} />
                <Text size="2">{CapabilitiesStrings.DETAIL.UNABLE_LOAD_CAPABILITY_TYPE_CONFIGURATION}</Text>
              </Box>
            ) : (
              <DynamicCapabilityForm
                capability={capability}
                capabilityType={capabilityType}
                isCreate={false}
                onSave={onSave}
                onCancel={onBack}
                loading={loading}
                error={error}
                canEdit={canUpdate}
                isEnabled={isEnabled}
                onEnabledChange={handleEnabledChange}
              />
            )}
          </Tabs.Content>

          {/* Status Tab */}
          <Tabs.Content value="status">
            <SettingsFormSection title={CommonStrings.LABELS.STATUS}>
              {capability.status ? (
                <Box
                  className="capability-detail__status-content"
                  dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(capability.status) }}
                />
              ) : (
                <Text className="capability-detail__no-status">{CapabilitiesStrings.DETAIL.NO_STATUS_INFORMATION_AVAILABLE}</Text>
              )}
            </SettingsFormSection>
          </Tabs.Content>

          {/* About Tab */}
          <Tabs.Content value="about">
            <SettingsFormSection title={CapabilitiesStrings.DETAIL.ABOUT}>
              {capabilityType?.about ? (
                <Box
                  className="capability-detail__about-content"
                  dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(capabilityType.about) }}
                />
              ) : (
                <Text className="capability-detail__no-about">{CapabilitiesStrings.DETAIL.NO_INFORMATION_AVAILABLE}</Text>
              )}
            </SettingsFormSection>
          </Tabs.Content>
        </Box>
      </Tabs.Root>

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={confirmAction === 'delete'}
        testId="capability-delete-dialog"
        onOpenChange={(open) => { if (!open) setConfirmAction(null); }}
        title={CapabilitiesStrings.DETAIL.DELETE_CAPABILITY}
        message={capability.deleteWarningMessage
          ? CapabilitiesStrings.DETAIL.ACTION_CANT_UNDONE(capability.deleteWarningMessage)
          : CapabilitiesStrings.DETAIL.PERMANENTLY_REMOVE_CAPABILITY_ITS_CONFIGURATION}
        confirmLabel={CommonStrings.ACTIONS.DELETE}
        variant="danger"
        onConfirm={() => {
          setConfirmAction(null);
          onDelete();
        }}
      />

      {/* Disable Confirmation Dialog */}
      <ConfirmDialog
        open={confirmAction === 'disable'}
        testId="capability-disable-dialog"
        onOpenChange={(open) => { if (!open) setConfirmAction(null); }}
        title={CapabilitiesStrings.DETAIL.DISABLE_CAPABILITY}
        message={capability.disableWarningMessage || CapabilitiesStrings.DETAIL.DISABLE_CONFIRM(capability.typeName)}
        confirmLabel={CapabilitiesStrings.DETAIL.DISABLE}
        variant="warning"
        onConfirm={handleDisableConfirm}
      />
    </Box>
  );
}

function SummaryItem({
  icon,
  label,
  value,
}: {
  icon?: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <Box className="capability-detail__summary-item">
      <Flex align="center" gap="2">
        {icon && <Box className="capability-detail__summary-icon">{icon}</Box>}
        <Text size="1" color="gray" className="capability-detail__summary-label">{label}:</Text>
      </Flex>
      <Text size="2" className="capability-detail__summary-value">{value}</Text>
    </Box>
  );
}

export default CapabilityDetail;
