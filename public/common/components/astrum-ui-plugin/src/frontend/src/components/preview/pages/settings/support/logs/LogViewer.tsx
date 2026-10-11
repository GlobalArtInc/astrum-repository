

import React, { useCallback } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Download, Stamp, Loader2 } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import {
  SettingsButton,
  SettingsTextInput,
  SettingsSelect,
  SettingsAlert,
} from '../../../../shared/form';
import { useLogViewer } from './useLogViewer';
import { REFRESH_RATES, LOG_SIZES } from './types';

import './LogViewer.scss';
import LogsStrings from '../../../../constants/pages/settings/support/LogsStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface LogViewerProps {
  filename: string;
  onBack?: () => void;
}

/**
 * LogViewer - Displays log file content with auto-refresh and mark insertion
 */
export function LogViewer({ filename, onBack }: LogViewerProps) {
  const {
    logContent,
    isLoading,
    error,
    mark,
    refreshPeriod,
    logSize,
    setMark,
    setRefreshPeriod,
    setLogSize,
    handleInsertMark,
    handleDownload,
    textareaRef,
  } = useLogViewer(filename);

  // filename comes from the route param (raw: true) and is already unencoded
  const isAstrumLog = filename === 'astrum.log';
  const canUpdate = ExtJS.checkPermission('astrum:logging:update');

  const handleMarkKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        handleInsertMark();
      }
    },
    [handleInsertMark]
  );

  return (
    <Box className="log-viewer">
      {/* Header with title and download button */}
      <Flex justify="between" align="center" className="log-viewer__header">
        <Text weight="medium" size="3" className="log-viewer__title">
          {LogsStrings.VIEWER.VIEWING(filename)}
        </Text>
        <SettingsButton variant="primary" onClick={handleDownload} icon={Download}>
          {CommonStrings.ACTIONS.DOWNLOAD}
        </SettingsButton>
      </Flex>

      {/* Toolbar */}
      <Box className="log-viewer__toolbar">
        <Flex align="center" justify="between" gap="4" wrap="wrap">
          {/* Mark insertion (only for astrum.log with update permission) */}
          {isAstrumLog && canUpdate && (
            <Flex align="center" gap="2" wrap="wrap" className="log-viewer__mark-section">
              <Text size="2" className="log-viewer__label">
                {LogsStrings.VIEWER.MARKER_INSERT}
              </Text>
              <SettingsTextInput
                value={mark}
                onChange={setMark}
                placeholder={LogsStrings.VIEWER.MARK_PLACEHOLDER}
                className="log-viewer__mark-input"
                onKeyDown={handleMarkKeyDown}
              />
              <SettingsButton
                variant="secondary"
                onClick={handleInsertMark}
                disabled={isLoading}
                icon={Stamp}
              >
                {LogsStrings.VIEWER.INSERT}
              </SettingsButton>
            </Flex>
          )}

          <Flex align="center" gap="4" wrap="wrap" className="log-viewer__options">
            {/* Refresh rate */}
            <Flex align="center" gap="2">
              <Text size="2" className="log-viewer__label">
                {LogsStrings.VIEWER.REFRESH_RATE}
              </Text>
              <SettingsSelect
                value={String(refreshPeriod)}
                onChange={setRefreshPeriod}
                options={REFRESH_RATES.map((r) => ({
                  value: String(r.value),
                  label: r.label,
                }))}
                className="log-viewer__select"
              />
            </Flex>

            {/* Log size */}
            <Flex align="center" gap="2">
              <Text size="2" className="log-viewer__label">
                {LogsStrings.VIEWER.SIZE}
              </Text>
              <SettingsSelect
                value={String(logSize)}
                onChange={setLogSize}
                options={LOG_SIZES.map((s) => ({
                  value: String(s.value),
                  label: s.label,
                }))}
                className="log-viewer__select"
              />
            </Flex>
          </Flex>
        </Flex>
      </Box>

      {/* Error alerts */}
      {error && (
        <Box mb="3">
          <SettingsAlert type="error">
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Log content */}
      <Box className="log-viewer__content">
        {isLoading ? (
          <Flex align="center" justify="center" className="log-viewer__loading">
            <Loader2 size={20} className="log-viewer__spinner" />
            <Text size="2" ml="2">
              {LogsStrings.VIEWER.LOADING_LOG_CONTENT}
            </Text>
          </Flex>
        ) : (
          <textarea
            ref={textareaRef}
            className="log-viewer__textarea"
            value={logContent}
            readOnly
            aria-label={LogsStrings.VIEWER.LOG_CONTENT(filename)}
          />
        )}
      </Box>
    </Box>
  );
}

export default LogViewer;
