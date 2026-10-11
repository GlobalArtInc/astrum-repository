

import React, { useState, useCallback } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2 } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsAlert } from '../../../../shared/form';
import { PageHeader } from '../../../../shared';
import { SupportZipForm } from './SupportZipForm';
import { SupportZipResponse } from './SupportZipResponse';
import { SupportZipHA } from './SupportZipHA';
import { useSupportZipApi } from './useSupportZipApi';
import {
  SupportZipParams,
  SupportZipResponse as SupportZipResponseType,
  DEFAULT_SUPPORT_ZIP_PARAMS,
} from './types';

import './SupportZipPage.scss';
import SupportZipStrings from '../../../../constants/pages/settings/support/SupportZipStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * SupportZipPage - Main Support ZIP generation page for Preview UI
 *
 * Handles both single-node and HA (clustered) support ZIP generation.
 *
 * Permission: astrum:atlas:create (to create ZIP), astrum:atlas:read (to view)
 * Route: #admin/support/supportzip
 */
export function SupportZipPage() {
  const [params, setParams] = useState<SupportZipParams>(DEFAULT_SUPPORT_ZIP_PARAMS);
  const [response, setResponse] = useState<SupportZipResponseType | null>(null);

  const { createSupportZip, loading, error, setError } = useSupportZipApi();

  // Permission checks
  const canRead = ExtJS.checkPermission('astrum:atlas:read');
  const canCreate = ExtJS.checkPermission('astrum:atlas:create');

  // Check if clustered (HA) mode
  const isClustered = ExtJS.useState(() => ExtJS.state().getValue('astrum.datastore.clustered.enabled'));

  // Handle single param change
  const handleParamChange = useCallback(
    (name: keyof SupportZipParams, value: boolean | number) => {
      setParams((prev) => ({
        ...prev,
        [name]: value,
      }));
    },
    []
  );

  // Handle create support ZIP (single-node)
  const handleSubmit = useCallback(async () => {
    try {
      setError(null);
      setResponse(null);
      const result = await createSupportZip(params);
      setResponse(result);
    } catch (_err) {
      // Error is handled by the hook
    }
  }, [params, createSupportZip, setError]);

  // Permission denied view
  if (!canRead) {
    return (
      <Box
        className="support-zip-page"
        data-testid="support-zip-page"
        data-permission="denied"
      >
        <PageHeader
          title={SupportZipStrings.PAGE.SUPPORT_ZIP}
          description={SupportZipStrings.PAGE.CREATES_ZIP_FILE_CONTAINING_USEFUL}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#admin/settings'; } },
            { label: SupportZipStrings.PAGE.SUPPORT_ZIP },
          ]}
        />

        <SettingsAlert type="warning" data-testid="support-zip-permission-warning">
          {SupportZipStrings.PAGE.YOU_DO_NOT_HAVE_PERMISSION}
        </SettingsAlert>
      </Box>
    );
  }

  // Render header
  const renderHeader = () => (
    <PageHeader
      title={SupportZipStrings.PAGE.SUPPORT_ZIP}
      description={SupportZipStrings.PAGE.CREATES_ZIP_FILE_CONTAINING_USEFUL}
      breadcrumbs={[
        { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#admin/settings'; } },
        { label: SupportZipStrings.PAGE.SUPPORT_ZIP },
      ]}
    />
  );

  // Render content based on state
  const renderContent = () => {
    // Loading state
    if (loading) {
      return (
        <Box className="support-zip-page__loading" data-testid="support-zip-loading" aria-live="polite" aria-busy="true">
          <Flex align="center" justify="center" gap="3" py="9">
            <Loader2 size={24} className="support-zip-page__spinner" aria-hidden="true" />
            <Text size="3">{SupportZipStrings.PAGE.CREATING_SUPPORT_ZIP}</Text>
          </Flex>
          <Text size="2" color="gray" align="center">
            {SupportZipStrings.PAGE.MAY_TAKE_FEW_MINUTES_COMPLETE}
          </Text>
        </Box>
      );
    }

    // Single ZIP response
    if (response) {
      return <SupportZipResponse response={response} />;
    }

    // For clustered mode, show HA component (self-contained)
    if (isClustered) {
      return (
        <SupportZipHA
          params={params}
          onParamChange={handleParamChange}
          disabled={loading || !canCreate}
        />
      );
    }

    // Default: show form
    return (
      <SupportZipForm
        params={params}
        onParamChange={handleParamChange}
        onSubmit={handleSubmit}
        isHa={false}
        disabled={loading || !canCreate}
      />
    );
  };

  return (
    <Box
      className="support-zip-page"
      data-testid="support-zip-page"
      data-loading={loading ? 'true' : 'false'}
      data-clustered={isClustered ? 'true' : 'false'}
    >
      {renderHeader()}

      {/* Permission warning for create */}
      {!canCreate && (
        <Box className="support-zip-page__alerts">
          <SettingsAlert type="warning" data-testid="support-zip-create-permission-warning">
            {SupportZipStrings.PAGE.YOU_DO_NOT_HAVE_PERMISSION_2}
          </SettingsAlert>
        </Box>
      )}

      {/* Error alert */}
      {error && (
        <Box className="support-zip-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)} data-testid="support-zip-error">
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      <Box className="support-zip-page__content">
        {renderContent()}
      </Box>
    </Box>
  );
}

export default SupportZipPage;
