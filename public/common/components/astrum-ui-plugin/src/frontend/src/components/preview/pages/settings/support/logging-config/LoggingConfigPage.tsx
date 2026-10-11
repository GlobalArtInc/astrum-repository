
import React from 'react';
import { Box, Flex } from '@radix-ui/themes';
import { Plus, RotateCcw } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsButton, SettingsAlert, ConfirmDialog } from '../../../../shared/form';
import { HelpSection, PageHeader } from '../../../../shared';
import { LoggersList } from './LoggersList';
import { LoggerForm } from './LoggerForm';
import { useLoggingConfig } from './useLoggingConfig';

import './LoggingConfigPage.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import LoggingStrings from '../../../../constants/pages/settings/support/LoggingStrings';

/**
 * LoggingConfigPage - Main Logging Configuration page for Preview UI
 *
 * Displays logger list with search/filter, and allows creating, editing, and resetting loggers.
 */
export function LoggingConfigPage() {
  const {
    viewMode,
    selectedLogger,
    deleteDialogOpen,
    resetAllDialogOpen,
    isDeleting,
    isResettingAll,
    error,
    refreshKey,
    handleSelectLogger,
    handleCreate,
    handleBack,
    handleSave,
    handleDeleteClick,
    handleDeleteConfirm,
    handleCancelDelete,
    handleResetAll,
    handleResetAllConfirm,
    handleCancelResetAll,
    clearError,
  } = useLoggingConfig();

  const canUpdate = ExtJS.checkPermission('astrum:logging:update');

  const navigateToSettings = () => {
    window.location.hash = '#preview/admin/settings';
  };

  const renderHeader = () => {
    if (viewMode === 'list') {
      const breadcrumbs = [
        { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
        { label: LoggingStrings.PAGE.LOGGING },
      ];
      const actions = canUpdate ? (
        <Flex gap="2">
          <SettingsButton variant="secondary" onClick={handleResetAll} disabled={isResettingAll} icon={RotateCcw}>
            {LoggingStrings.PAGE.RESET_DEFAULT_LEVELS}
          </SettingsButton>
          <SettingsButton variant="primary" onClick={handleCreate} icon={Plus}>
            {LoggingStrings.PAGE.CREATE_LOGGER}
          </SettingsButton>
        </Flex>
      ) : undefined;
      return (
        <PageHeader
          title={LoggingStrings.PAGE.LOGGING}
          description={LoggingStrings.PAGE.CONTROL_LOGGING_LEVELS}
          breadcrumbs={breadcrumbs}
          actions={actions}
          className="logging-config-page__header"
        />
      );
    }

    // A logger is titled by its name, which its form shows again in the Logger Name field
    const title = viewMode === 'create'
      ? LoggingStrings.PAGE.CREATE_LOGGER
      : selectedLogger || LoggingStrings.PAGE.EDIT_LOGGER;
    const breadcrumbs = [
      { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
      { label: LoggingStrings.PAGE.LOGGING, onClick: handleBack },
      { label: viewMode === 'create' ? CommonStrings.ACTIONS.CREATE : selectedLogger || LoggingStrings.PAGE.LOGGER },
    ];

    return (
      <PageHeader
        title={title}
        description={viewMode === 'detail' ? LoggingStrings.PAGE.EDIT_LOGGER : undefined}
        breadcrumbs={breadcrumbs}
        className="logging-config-page__header"
      />
    );
  };

  return (
    <Box className="logging-config-page" data-testid="logging-config-page">
      {renderHeader()}

      {/* Alerts */}
      {error && (
        <Box className="logging-config-page__alerts">
          <SettingsAlert type="error" onClose={clearError}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      <Box className="logging-config-page__content">
        {viewMode === 'list' && <LoggersList key={refreshKey} onSelect={handleSelectLogger} />}

        {viewMode === 'create' && (
          <LoggerForm isCreate={true} onSave={handleSave} onCancel={handleBack} />
        )}

        {viewMode === 'detail' && selectedLogger && (
          <LoggerForm
            loggerName={selectedLogger}
            onSave={handleSave}
            onCancel={handleBack}
            onDelete={canUpdate ? handleDeleteClick : undefined}
          />
        )}
      </Box>

      {/* Help Section - only shown in list view */}
      {viewMode === 'list' && (
        <HelpSection
          title={LoggingStrings.PAGE.ABOUT_LOGGING_CONFIGURATION}
          content={LoggingStrings.PAGE.CONFIGURE_LOGGING_LEVELS_DIFFERENT_PACKAGES}
          docLink={{
            label: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
            href: 'https://help.globalart.dev/en/logging-configuration.html',
          }}
        />
      )}

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        testId="delete-logger-override-dialog"
        onOpenChange={(open) => !open && handleCancelDelete()}
        title={LoggingStrings.PAGE.DELETE_LOGGER_OVERRIDE}
        message={LoggingStrings.PAGE.REMOVE_CUSTOM_LOG_LEVEL_LOGGER(selectedLogger)}
        confirmLabel={isDeleting ? LoggingStrings.PAGE.DELETING : CommonStrings.ACTIONS.DELETE}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="danger"
        onConfirm={handleDeleteConfirm}
      />

      {/* Reset All Confirmation Dialog */}
      <ConfirmDialog
        open={resetAllDialogOpen}
        testId="reset-all-loggers-dialog"
        onOpenChange={(open) => !open && handleCancelResetAll()}
        title={LoggingStrings.PAGE.RESET_ALL_LOGGERS}
        message={LoggingStrings.PAGE.YOU_SURE_YOU_WANT_RESET}
        confirmLabel={LoggingStrings.PAGE.RESET_ALL}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="warning"
        onConfirm={handleResetAllConfirm}
      />
    </Box>
  );
}

export default LoggingConfigPage;
