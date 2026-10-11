

import React from 'react';
import { Box, Flex, Text, } from '@radix-ui/themes';
import { Archive, Server } from 'lucide-react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsCheckbox,
  SettingsSelect,
  SettingsButton,
} from '../../../../shared/form';
import { SupportZipParams, ARCHIVED_LOG_OPTIONS } from './types';

import './SupportZipForm.scss';
import SupportZipStrings from '../../../../constants/pages/settings/support/SupportZipStrings';

interface SupportZipFormProps {
  params: SupportZipParams;
  onParamChange: (name: keyof SupportZipParams, value: boolean | number) => void;
  onSubmit: () => void;
  onSubmitAll?: () => void;
  isHa?: boolean;
  disabled?: boolean;
  hideActions?: boolean;
}

/**
 * SupportZipForm - Form for selecting support ZIP contents
 */
export function SupportZipForm({
  params,
  onParamChange,
  onSubmit,
  onSubmitAll,
  isHa = false,
  disabled = false,
  hideActions = false,
}: SupportZipFormProps) {
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit();
  };

  return (
    <Box className="support-zip-form">
      {/* Description */}
      <Box className="support-zip-form__description" mb="4">
        <Text as="p" size="2" color="gray">
          {SupportZipStrings.FORM.NO_INFORMATION_SENT_GLOBALART_WHEN}
        </Text>
        <Text as="p" size="2" color="gray" mt="2">
          {SupportZipStrings.FORM.SUPPORT_ZIP_CREATION_MAY_TAKE}
        </Text>
      </Box>

      <SettingsForm testId="support-zip-form" onSubmit={handleSubmit} showActions={false} pristine={false} noDirtyTracking>
        {/* Contents Section */}
        <SettingsFormSection title={SupportZipStrings.FORM.CONTENTS}>
          <Box className="support-zip-form__checkboxes">
            <SettingsCheckbox
              name="systemInformation"
              label={SupportZipStrings.FORM.SYSTEM_INFORMATION_REPORT}
              checked={params.systemInformation}
              onChange={(checked) => onParamChange('systemInformation', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-systemInformation"
            />
            <SettingsCheckbox
              name="threadDump"
              label={SupportZipStrings.FORM.JVM_THREAD_DUMP}
              checked={params.threadDump}
              onChange={(checked) => onParamChange('threadDump', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-threadDump"
            />
            <SettingsCheckbox
              name="configuration"
              label={SupportZipStrings.FORM.CONFIGURATION_FILES}
              checked={params.configuration}
              onChange={(checked) => onParamChange('configuration', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-configuration"
            />
            <SettingsCheckbox
              name="security"
              label={SupportZipStrings.FORM.SECURITY_CONFIGURATION_FILES}
              checked={params.security}
              onChange={(checked) => onParamChange('security', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-security"
            />
            <SettingsCheckbox
              name="log"
              label={SupportZipStrings.FORM.LOG_FILES}
              checked={params.log}
              onChange={(checked) => onParamChange('log', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-log"
            />
            <SettingsCheckbox
              name="taskLog"
              label={SupportZipStrings.FORM.TASK_LOG_FILES}
              checked={params.taskLog}
              onChange={(checked) => onParamChange('taskLog', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-taskLog"
            />
            <SettingsCheckbox
              name="replication"
              label={SupportZipStrings.FORM.REPLICATION_LOG_FILES}
              checked={params.replication}
              onChange={(checked) => onParamChange('replication', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-replication"
            />
            <SettingsCheckbox
              name="auditLog"
              label={SupportZipStrings.FORM.AUDIT_LOG_FILES}
              checked={params.auditLog}
              onChange={(checked) => onParamChange('auditLog', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-auditLog"
            />
            <SettingsCheckbox
              name="metrics"
              label={SupportZipStrings.FORM.SYSTEM_COMPONENT_METRICS}
              checked={params.metrics}
              onChange={(checked) => onParamChange('metrics', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-metrics"
            />
            <SettingsCheckbox
              name="jmx"
              label={SupportZipStrings.FORM.JMX_INFORMATION}
              checked={params.jmx}
              onChange={(checked) => onParamChange('jmx', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-jmx"
            />

            {/* Archived logs dropdown */}
            <Box mt="3">
              <SettingsSelect
                name="archivedLog"
                label={SupportZipStrings.FORM.INCLUDE_LOGS_PREVIOUS_DAYS}
                value={String(params.archivedLog)}
                onChange={(value) => onParamChange('archivedLog', Number(value))}
                options={ARCHIVED_LOG_OPTIONS.map((opt) => ({
                  value: String(opt.value),
                  label: opt.label,
                }))}
                disabled={disabled}
                data-testid="support-zip-select-archivedLog"
              />
            </Box>
          </Box>
        </SettingsFormSection>

        {/* Options Section */}
        <SettingsFormSection title={SupportZipStrings.FORM.OPTIONS}>
          <Box className="support-zip-form__checkboxes">
            <SettingsCheckbox
              name="limitFileSizes"
              label={SupportZipStrings.FORM.LIMIT_FILES_ZIP_ARCHIVE_30}
              checked={params.limitFileSizes}
              onChange={(checked) => onParamChange('limitFileSizes', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-limitFileSizes"
            />
            <SettingsCheckbox
              name="limitZipSize"
              label={SupportZipStrings.FORM.LIMIT_ZIP_ARCHIVE_50_MB}
              checked={params.limitZipSize}
              onChange={(checked) => onParamChange('limitZipSize', checked)}
              disabled={disabled}
              data-testid="support-zip-checkbox-limitZipSize"
            />
          </Box>
        </SettingsFormSection>

        {/* Actions */}
        {!hideActions && (
          <Flex gap="3" mt="4">
            <SettingsButton
              type="submit"
              variant="primary"
              disabled={disabled}
              icon={Archive}
              data-testid="support-zip-create-button"
              data-analytics-id="as-support-zip-create"
            >
              {SupportZipStrings.FORM.CREATE_SUPPORT_ZIP}
            </SettingsButton>
            {isHa && onSubmitAll && (
              <SettingsButton
                type="button"
                variant="primary"
                onClick={onSubmitAll}
                disabled={disabled}
                icon={Server}
                data-testid="support-zip-create-all-button"
                data-analytics-id="as-support-zip-create-all"
              >
                {SupportZipStrings.FORM.CREATE_SUPPORT_ZIP_ALL_NODES}
              </SettingsButton>
            )}
          </Flex>
        )}
      </SettingsForm>
    </Box>
  );
}

export default SupportZipForm;
