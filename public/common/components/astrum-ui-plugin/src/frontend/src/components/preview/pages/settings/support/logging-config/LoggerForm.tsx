

import React, { useCallback } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2 } from 'lucide-react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsSelect,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import { useLoggerForm } from './useLoggerForm';
import { LOG_LEVELS } from './types';

import './LoggerForm.scss';
import LoggingStrings from '../../../../constants/pages/settings/support/LoggingStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface LoggerFormProps {
  loggerName?: string | null;
  isCreate?: boolean;
  onSave: () => void;
  onCancel: () => void;
  onDelete?: () => void;
}

/**
 * LoggerForm - Create or edit a logger configuration
 */
export function LoggerForm({ loggerName, isCreate = false, onSave, onCancel, onDelete }: LoggerFormProps) {
  const { name, level, isDirty, isLoading, isSaving, error, setName, setLevel, handleSubmit } = useLoggerForm({
    loggerName,
    isCreate,
    onSave,
    onCancel,
  });

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        handleSubmit();
      }
    },
    [handleSubmit]
  );

  if (isLoading) {
    return (
      <Box className="logger-form logger-form--loading">
        <Flex align="center" justify="center" gap="2" py="9">
          <Loader2 size={20} className="logger-form__spinner" />
          <Text size="2">{LoggingStrings.FORM.LOADING_LOGGER}</Text>
        </Flex>
      </Box>
    );
  }

  return (
    <Box className="logger-form">
      {/* Error alert */}
      {error && (
        <Box mb="4">
          <SettingsAlert type="error">{error}</SettingsAlert>
        </Box>
      )}

      <SettingsForm
        testId="logger-form"
        onSubmit={handleSubmit}
        onCancel={onCancel}
        dirty={isDirty}
        loading={isSaving}
        submitLabel={isCreate ? LoggingStrings.FORM.CREATE_LOGGER : CommonStrings.ACTIONS.SAVE}
        footerExtra={
          !isCreate && onDelete ? (
            <SettingsButton type="button" variant="danger" onClick={onDelete} disabled={isSaving} testId="form-delete">
              {CommonStrings.ACTIONS.DELETE}
            </SettingsButton>
          ) : undefined
        }
      >
        <SettingsFormSection>
          {/* Logger Name */}
          <Box mb="4">
            <SettingsTextInput
              name="loggerName"
              label={LoggingStrings.FORM.LOGGER_NAME}
              value={name}
              onChange={setName}
              placeholder="e.g., org.globalart.astrum"
              helpText={LoggingStrings.FORM.FULLY_QUALIFIED_LOGGER_NAME_E}
              disabled={!isCreate}
              required
              onKeyDown={handleKeyDown}
            />
            {isCreate && (
              <Text size="1" color="gray" mt="1">
                {LoggingStrings.FORM.ENTER_FULLY_QUALIFIED_LOGGER_NAME}
              </Text>
            )}
          </Box>

          {/* Logger Level */}
          <Box mb="4">
            <SettingsSelect
              name="loggerLevel"
              label={LoggingStrings.FORM.LOGGER_LEVEL}
              value={level}
              onChange={setLevel}
              options={LOG_LEVELS.map((l) => ({ value: l, label: l }))}
              helpText={LoggingStrings.FORM.LOG_LEVEL_ERROR_LEAST_VERBOSE}
            />
            <Text size="1" color="gray" mt="1">
              {LoggingStrings.FORM.OVERRIDES_INHERITED_LEVEL}
              {!isCreate && ` ${LoggingStrings.FORM.DELETE_TO_REVERT}`}
            </Text>
          </Box>
        </SettingsFormSection>
      </SettingsForm>
    </Box>
  );
}

export default LoggerForm;
