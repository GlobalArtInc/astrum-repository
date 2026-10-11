
import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { Box, Text, } from '@radix-ui/themes';
import { Lock, } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import {
  PageHeader,
  LoadingState,
  ErrorState,
  HelpSection,
  useToast,
} from '../../../../shared';
import {
  SettingsForm,
  SettingsFormSection,
  SettingsTransferList,
} from '../../../../shared/form';
import { useProprietaryApi } from './useProprietaryApi';
import { RepositoryReference } from './types';

import './ProprietaryPage.scss';
import ProprietaryStrings from '../../../../constants/pages/settings/repository/ProprietaryStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

/**
 * ProprietaryPage - Proprietary Repositories settings page for Preview UI
 *
 * Allows administrators to mark hosted repositories as containing proprietary components.
 */
export function ProprietaryPage() {
  const [possibleRepos, setPossibleRepos] = useState<RepositoryReference[]>([]);
  const [enabledRepos, setEnabledRepos] = useState<string[]>([]);
  const [initialEnabledRepos, setInitialEnabledRepos] = useState<string[]>([]);
  const [loadingData, setLoadingData] = useState(true);

  // Toast notifications (app-level provider)
  const toast = useToast();

  const {
    loading,
    error,
    setError,
    fetchSettings,
    fetchPossibleRepositories,
    updateSettings,
  } = useProprietaryApi();

  const canUpdate = ExtJS.checkPermission('astrum:settings:update');

  // Load data on mount
  const loadData = useCallback(async () => {
    setLoadingData(true);
    setError(null);
    try {
      const [settings, repos] = await Promise.all([
        fetchSettings(),
        fetchPossibleRepositories(),
      ]);
      setPossibleRepos(repos);
      setEnabledRepos(settings.enabledRepositories);
      setInitialEnabledRepos(settings.enabledRepositories);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoadingData(false);
    }
  }, [fetchSettings, fetchPossibleRepositories, setError]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Check if form has changes
  const isDirty = useMemo(() => {
    if (enabledRepos.length !== initialEnabledRepos.length) return true;
    const sortedCurrent = [...enabledRepos].sort();
    const sortedInitial = [...initialEnabledRepos].sort();
    return sortedCurrent.some((repo, idx) => repo !== sortedInitial[idx]);
  }, [enabledRepos, initialEnabledRepos]);

  // Convert to transfer list items - use 'name' as that's what SettingsTransferList expects
  const availableItems = useMemo(() => {
    return possibleRepos.map(repo => ({
      id: repo.id,
      name: repo.name,
    }));
  }, [possibleRepos]);

  // Convert selected IDs to full objects for consistency with SettingsTransferList
  const selectedItemObjects = useMemo(() => {
    return enabledRepos.map(id => {
      const repo = possibleRepos.find(r => r.id === id);
      return { id, name: repo?.name || id };
    });
  }, [enabledRepos, possibleRepos]);

  // Handle transfer list changes - extract IDs from the selected items
  const handleTransferChange = useCallback((selectedItems: Array<{ id: string; name: string }>) => {
    const ids = selectedItems.map(item => item.id);
    setEnabledRepos(ids);
  }, []);

  const handleSave = useCallback(async () => {
      const result = await updateSettings(enabledRepos);
      setInitialEnabledRepos(result.enabledRepositories);
      setEnabledRepos(result.enabledRepositories);
      toast.success(ProprietaryStrings.PAGE.PROPRIETARY_REPOSITORIES_SETTINGS_SAVED_SUCCESSFULLY);
  }, [enabledRepos, updateSettings, toast]);

  const handleDiscard = useCallback(() => {
    setEnabledRepos(initialEnabledRepos);
  }, [initialEnabledRepos]);

  // Read-only view for users without update permission
  const renderReadOnly = () => (
    <Box className="proprietary-page__read-only">
      <Text size="2" className="proprietary-page__read-only-label">
        {ProprietaryStrings.PAGE.ENABLED_PROPRIETARY_REPOSITORIES}
      </Text>
      {enabledRepos.length > 0 ? (
        <Box className="proprietary-page__read-only-list">
          {enabledRepos.map(repo => (
            <Text key={repo} size="2" className="proprietary-page__read-only-item">
              {repo}
            </Text>
          ))}
        </Box>
      ) : (
        <Text size="2" className="proprietary-page__read-only-empty">
          {ProprietaryStrings.PAGE.NO_PROPRIETARY_REPOSITORIES_CONFIGURED}
        </Text>
      )}
    </Box>
  );

  // Show loading state
  if (loadingData) {
    return (
      <Box className="proprietary-page">
        <PageHeader
          icon={Lock}
          title={ProprietaryStrings.PAGE.PROPRIETARY_REPOSITORIES}
          description={ProprietaryStrings.PAGE.CONFIGURE_WHICH_REPOSITORIES_CONTAIN_PROPRIETARY}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
            { label: ProprietaryStrings.PAGE.PROPRIETARY_REPOSITORIES }
          ]}
        />
        <LoadingState message={ProprietaryStrings.PAGE.LOADING_SETTINGS} />
      </Box>
    );
  }

  // Show error state with retry
  if (error && !possibleRepos.length) {
    return (
      <Box className="proprietary-page">
        <PageHeader
          icon={Lock}
          title={ProprietaryStrings.PAGE.PROPRIETARY_REPOSITORIES}
          description={ProprietaryStrings.PAGE.CONFIGURE_WHICH_REPOSITORIES_CONTAIN_PROPRIETARY}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
            { label: ProprietaryStrings.PAGE.PROPRIETARY_REPOSITORIES }
          ]}
        />
        <ErrorState
          title={ProprietaryStrings.PAGE.FAILED_LOAD}
          message={error}
          onRetry={loadData}
        />
      </Box>
    );
  }

  return (
    <Box className="proprietary-page">
      <PageHeader
        icon={Lock}
        title={ProprietaryStrings.PAGE.PROPRIETARY_REPOSITORIES}
        description={ProprietaryStrings.PAGE.CONFIGURE_WHICH_REPOSITORIES_CONTAIN_PROPRIETARY}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
          { label: ProprietaryStrings.PAGE.PROPRIETARY_REPOSITORIES }
        ]}
      />

      {/* Error Alert */}
      {error && (
        <Box className="proprietary-page__alerts">
          <ErrorState
            variant="inline"
            title={ProprietaryStrings.PAGE.ERROR}
            message={error}
            onRetry={() => setError(null)}
            retryText={ProprietaryStrings.PAGE.DISMISS}
          />
        </Box>
      )}

      {/* Content */}
      <Box className="proprietary-page__content">
        <SettingsForm 
          testId="proprietary-form"
          onSubmit={handleSave} 
          onCancel={handleDiscard} 
          showHeader={false} 
          dirty={isDirty}
          cancelDisabled={!isDirty}
        >
          {/* Help Section - Using standard HelpSection component */}
          <HelpSection
            title={ProprietaryStrings.PAGE.ABOUT_PROPRIETARY_COMPONENTS}
            content={ProprietaryStrings.PAGE.MARKING_REPOSITORY_CONTAINING_PROPRIETARY_COMPONENTS}
            docLink={{
              label: CommonStrings.ACTIONS.LEARN_MORE,
              href: "https://help.globalart.dev/en/proprietary-component-configuration.html",
            }}
          />

          <SettingsFormSection title={ProprietaryStrings.PAGE.REPOSITORY_SELECTION}>
            {canUpdate ? (
              <SettingsTransferList
                name="proprietaryRepositories"
                testId="proprietary-repos"
                availableItems={availableItems}
                selectedItems={selectedItemObjects}
                onChange={handleTransferChange}
                availableLabel={ProprietaryStrings.PAGE.GENERIC_HOSTED_REPOSITORIES}
                selectedLabel={ProprietaryStrings.PAGE.PROPRIETARY_HOSTED_REPOSITORIES}
                disabled={loading}
              />
            ) : (
              renderReadOnly()
            )}
          </SettingsFormSection>

          {/* Actions are in the sticky header bar via SettingsForm */}
        </SettingsForm>
      </Box>
    </Box>
  );
}

export default ProprietaryPage;
