

import React from 'react';
import { Box, Flex, Text, Table } from '@radix-ui/themes';
import { Database, AlertCircle } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import { PageHeader } from '../../../../shared/PageHeader';
import { LoadingState } from '../../../../shared/LoadingState';
import { ErrorState } from '../../../../shared/ErrorState';
import { useDataStoreForm } from './useDataStoreForm';
import { JdbcParameterEditor, ParameterValidation } from './JdbcParameterEditor';

import './DataStorePage.scss';
import DataStoreStrings from '../../../../constants/pages/settings/repository/DataStoreStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

/**
 * DataStorePage - DataStore Configuration settings page for Preview UI
 *
 * Structured as:
 * 1. Connection Overview (read-only)
 * 2. Connection Pool Settings
 * 3. Advanced JDBC Parameters (key-value editor)
 * 4. Effective Configuration Preview
 * 5. Save / Discard actions
 */
export function DataStorePage() {
  const {
    data,
    field,
    isLoading,
    isSaving,
    isPristine,
    saveError,
    loadError,
    databaseType,
    effectiveConfig,
    parameterValidations,
    hasParameterErrors,
    canSave,
    showAllValidation,
    setParameters,
    setMaxPool,
    showResetConfirm,
    requestResetParams,
    confirmResetParams,
    cancelResetParams,
    submit,
    reset,
  } = useDataStoreForm();

  const canUpdate = ExtJS.checkPermission('astrum:settings:update');
  const error = saveError || loadError;

  return (
    <Box className="datastore-page">
      {/* Header - using shared PageHeader for consistent UX */}
      <PageHeader
        icon={Database}
        title={DataStoreStrings.PAGE.DATA_STORE}
        description={DataStoreStrings.PAGE.CONFIGURE_CONNECTION_USED_DATABASE}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
          { label: DataStoreStrings.PAGE.DATA_STORE }
        ]}
      />

      {/* Error Alert */}
      {error && (
        <Box className="datastore-page__alerts">
          <ErrorState
            variant="inline"
            title={DataStoreStrings.PAGE.ERROR}
            message={error}
          />
        </Box>
      )}

      {/* Reset Confirmation Dialog */}
      {showResetConfirm && (
        <Box className="datastore-page__alerts">
          <SettingsAlert type="warning">
            <Flex justify="between" align="center" gap="3">
              <Text size="2">
                {DataStoreStrings.PAGE.RESET_ALL_ADVANCED_JDBC_PARAMETERS}
              </Text>
              <Flex gap="2">
                <SettingsButton variant="secondary" onClick={cancelResetParams}>
                  {CommonStrings.ACTIONS.CANCEL}
                </SettingsButton>
                <SettingsButton variant="danger" onClick={confirmResetParams}>
                  {DataStoreStrings.PAGE.RESET_DEFAULTS}
                </SettingsButton>
              </Flex>
            </Flex>
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      {isLoading ? (
        <LoadingState message={DataStoreStrings.PAGE.LOADING_CONFIGURATION} />
      ) : !loadError ? (
        <SettingsForm
          testId="datastore-form"
          onSubmit={submit}
          onCancel={reset}
          loading={isSaving}
          pristine={isPristine}
          submitDisabled={!(canUpdate && canSave)}
        >
          {/* 1. Connection Overview (Read-Only) */}
          <SettingsFormSection title={DataStoreStrings.PAGE.CONNECTION_OVERVIEW}>
            <Box className="datastore-page__readonly-fields">
              <Box className="datastore-page__readonly-field">
                <Text size="2" weight="medium" className="datastore-page__readonly-label">
                  {DataStoreStrings.PAGE.DATABASE_TYPE}
                </Text>
                <Text size="2" className="datastore-page__readonly-value">
                  {databaseType}
                </Text>
              </Box>
              <Box className="datastore-page__readonly-field">
                <Text size="2" weight="medium" className="datastore-page__readonly-label">
                  {DataStoreStrings.PAGE.JDBC_URL}
                </Text>
                <Text size="2" className="datastore-page__readonly-value datastore-page__readonly-value--mono">
                  {data.jdbcUrl || DataStoreStrings.PAGE.NOT_CONFIGURED}
                </Text>
              </Box>
              <Box className="datastore-page__readonly-field">
                <Text size="2" weight="medium" className="datastore-page__readonly-label">
                  {DataStoreStrings.PAGE.SCHEMA}
                </Text>
                <Text size="2" className="datastore-page__readonly-value">
                  {data.schema || DataStoreStrings.PAGE.NOT_CONFIGURED}
                </Text>
              </Box>
              <Box className="datastore-page__readonly-field">
                <Text size="2" weight="medium" className="datastore-page__readonly-label">
                  {DataStoreStrings.PAGE.USERNAME}
                </Text>
                <Text size="2" className="datastore-page__readonly-value">
                  {data.username || DataStoreStrings.PAGE.NOT_CONFIGURED}
                </Text>
              </Box>
            </Box>
          </SettingsFormSection>

          {/* 2. Connection Pool Settings */}
          <SettingsFormSection title={DataStoreStrings.PAGE.CONNECTION_POOL_SETTINGS}>
            <SettingsTextInput
              name="maximumConnectionPool"
              label={DataStoreStrings.PAGE.MAXIMUM_CONNECTION_POOL_SIZE}
              value={String(data.maximumConnectionPool)}
              onChange={setMaxPool}
              type="number"
              min={1}
              max={3000}
              helpText={DataStoreStrings.PAGE.MAXIMUM_NUMBER_DATABASE_CONNECTIONS_POOL}
              error={field('maximumConnectionPool').error}
              disabled={!canUpdate || isSaving}
            />
          </SettingsFormSection>

          {/* 3. Advanced JDBC Parameters */}
          <SettingsFormSection
            title={DataStoreStrings.PAGE.ADVANCED_JDBC_PARAMETERS}
            description={DataStoreStrings.PAGE.CONFIGURE_ADDITIONAL_JDBC_CONNECTION_PARAMETERS}
          >
            {showAllValidation && hasParameterErrors && (
              <Flex align="center" gap="2" className="datastore-page__section-error">
                <AlertCircle size={16} />
                <Text size="2">{DataStoreStrings.PAGE.FIX_PARAMETER_ERRORS_BEFORE_SAVING}</Text>
              </Flex>
            )}
            <JdbcParameterEditor
              parameters={data.jdbcParameters}
              onChange={setParameters}
              showAllValidation={showAllValidation}
              onReset={requestResetParams}
              disabled={!canUpdate || isSaving}
              validations={parameterValidations as ParameterValidation[]}
            />
          </SettingsFormSection>

          {/* 4. Effective Configuration Preview */}
          {effectiveConfig.length > 0 && (
            <SettingsFormSection
              title={DataStoreStrings.PAGE.EFFECTIVE_JDBC_CONFIGURATION}
              description={DataStoreStrings.PAGE.PREVIEW_FINAL_RESOLVED_PARAMETER_LIST}
            >
              <Box className="datastore-page__effective-config">
                <Table.Root>
                  <Table.Header>
                    <Table.Row>
                      <Table.ColumnHeaderCell>{DataStoreStrings.PAGE.PARAMETER}</Table.ColumnHeaderCell>
                      <Table.ColumnHeaderCell>{DataStoreStrings.PAGE.VALUE}</Table.ColumnHeaderCell>
                      <Table.ColumnHeaderCell>{DataStoreStrings.PAGE.SOURCE}</Table.ColumnHeaderCell>
                    </Table.Row>
                  </Table.Header>
                  <Table.Body>
                    {effectiveConfig.map((param) => (
                      <Table.Row key={param.name}>
                        <Table.Cell>
                          <Text size="2" className="datastore-page__effective-name">
                            {param.name}
                          </Text>
                        </Table.Cell>
                        <Table.Cell>
                          <Text size="2" className="datastore-page__effective-value">
                            {param.value}
                          </Text>
                        </Table.Cell>
                        <Table.Cell>
                          <Text
                            size="1"
                            className={`datastore-page__effective-source datastore-page__effective-source--${param.source.toLowerCase()}`}
                          >
                            {param.source === 'Custom' ? DataStoreStrings.PAGE.SOURCE_CUSTOM : DataStoreStrings.PAGE.SOURCE_DEFAULT}
                          </Text>
                        </Table.Cell>
                      </Table.Row>
                    ))}
                  </Table.Body>
                </Table.Root>
              </Box>
            </SettingsFormSection>
          )}

          {/* Actions are in the sticky header bar via SettingsForm */}
        </SettingsForm>
      ) : (
        <ErrorState
          title={DataStoreStrings.PAGE.FAILED_LOAD_CONFIGURATION}
          message={DataStoreStrings.PAGE.UNABLE_LOAD_DATA_STORE_CONFIGURATION}
          onRetry={() => window.location.reload()}
        />
      )}
    </Box>
  );
}

export default DataStorePage;
