
import React, { useCallback } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { AlertTriangle } from 'lucide-react';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsAlert } from '../../../../shared/form';
import { HelpSection, PageHeader } from '../../../../shared';
import { LogsList } from './LogsList';
import { LogViewer } from './LogViewer';

import './LogsPage.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import LogsStrings from '../../../../constants/pages/settings/support/LogsStrings';

const ROUTE_LIST = 'preview.admin.support.logs.list';
const ROUTE_DETAIL = 'preview.admin.support.logs.detail';

/**
 * LogsPage - Main Logs management page for Preview UI
 *
 * Displays log file list and allows viewing individual log files.
 * Note: Logs are not available via the UI in clustered (HA) mode.
 *
 * The selected log file is stored in the route URL (/*filename param with
 * raw: true so task log paths like tasks/foo.log are preserved). This means
 * the global header Refresh button — which calls router.stateService.reload()
 * — remounts the component with the same filename param and keeps the viewer
 * open instead of navigating back to the list.
 */
export function LogsPage() {
  const router = useRouter();
  const { params } = useCurrentStateAndParams();

  // filename is the raw path from the URL; null/undefined when on the list route
  const selectedLog: string | null = (params.filename as string) ?? null;
  const viewMode = selectedLog ? 'detail' : 'list';

  // Permission check
  const canRead = ExtJS.checkPermission('astrum:logging:read');

  // Check if clustered (HA) mode
  const isClustered = ExtJS.state().getValue('astrum.datastore.clustered.enabled');

  const handleSelectLog = useCallback((filename: string) => {
    router.stateService.go(ROUTE_DETAIL, {filename});
  }, [router.stateService.go]);

  const handleBack = useCallback(() => {
    router.stateService.go(ROUTE_LIST);
  }, [router.stateService.go]);

  // Navigation helper for Settings breadcrumb
  const navigateToSettings = () => {
    window.location.hash = '#preview/admin/settings';
  };

  // Render header based on view mode
  const renderHeader = () => {
    if (viewMode === 'list') {
      const breadcrumbs = [
        { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
        { label: LogsStrings.PAGE.LOGS },
      ];
      return (
        <PageHeader
          title={LogsStrings.PAGE.LOGS}
          description={LogsStrings.PAGE.VIEW_CURRENT_LOG_CONTENTS}
          breadcrumbs={breadcrumbs}
          className="logs-page__header"
        />
      );
    }

    const decodedLog = selectedLog ?? '';
    const breadcrumbs = [
      { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
      { label: LogsStrings.PAGE.LOGS, onClick: handleBack },
      { label: decodedLog },
    ];

    return (
      <PageHeader
        title={LogsStrings.PAGE.LOG_VIEWER}
        description={decodedLog}
        breadcrumbs={breadcrumbs}
        className="logs-page__header"
      />
    );
  };

  // No permission state
  if (!canRead) {
    const breadcrumbs = [
      { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
      { label: LogsStrings.PAGE.LOGS },
    ];
    return (
      <Box className="logs-page" data-testid="logs-page">
        <PageHeader
          title={LogsStrings.PAGE.LOGS}
          description={LogsStrings.PAGE.VIEW_CURRENT_LOG_CONTENTS}
          breadcrumbs={breadcrumbs}
          className="logs-page__header"
        />

        <SettingsAlert type="warning">
          {LogsStrings.PAGE.YOU_DO_NOT_HAVE_PERMISSION}
        </SettingsAlert>
      </Box>
    );
  }

  return (
    <Box className="logs-page" data-testid="logs-page">
      {renderHeader()}

      {/* Clustered mode warning */}
      {isClustered && (
        <Box className="logs-page__clustered-warning" mb="4">
          <SettingsAlert type="warning">
            <Flex align="center" gap="2">
              <AlertTriangle size={16} />
              <Text>
                {LogsStrings.PAGE.LOGS_NOT_AVAILABLE_VIA_UI}
              </Text>
            </Flex>
          </SettingsAlert>
        </Box>
      )}

      {/* Main content - only shown when not in clustered mode */}
      {!isClustered && (
        <Box className="logs-page__content">
          {viewMode === 'list' && <LogsList onSelect={handleSelectLog} />}

          {viewMode === 'detail' && selectedLog && (
            <LogViewer filename={selectedLog} onBack={handleBack} />
          )}
        </Box>
      )}

      {/* Help Section - only shown in list view */}
      {viewMode === 'list' && (
        <HelpSection
          title={LogsStrings.PAGE.ABOUT_LOGS}
          content={LogsStrings.PAGE.VIEW_DOWNLOAD_LOG_FILES_ASTRUM}
          docLink={{
            label: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
            href: 'https://help.globalart.dev/en/logging-configuration.html',
          }}
        />
      )}
    </Box>
  );
}

export default LogsPage;

