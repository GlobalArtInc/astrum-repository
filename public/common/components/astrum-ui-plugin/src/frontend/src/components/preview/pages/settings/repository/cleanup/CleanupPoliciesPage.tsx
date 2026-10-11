

import React, { useState, useEffect, useCallback } from 'react';
import { Box } from '@radix-ui/themes';
import { Plus } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { useToast, LoadingState, PageHeader } from '../../../../shared';
import { SettingsButton, SettingsAlert, ConfirmDialog } from '../../../../shared/form';
import { CleanupPoliciesList } from './CleanupPoliciesList';
import { CleanupPolicyForm } from './CleanupPolicyForm';
import { useCleanupPoliciesApi } from './useCleanupPoliciesApi';
import { CleanupPolicy, CleanupPolicyFormData, FormatCriteria, formatDisplayName } from './types';

import './CleanupPoliciesPage.scss';
import CleanupPoliciesStrings from '../../../../constants/pages/settings/repository/CleanupPoliciesStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

// Base path for cleanup policies URLs
const BASE_PATH = 'admin/repository/cleanup-policies';

/**
 * URL-based routing patterns:
 * - /cleanup-policies           → List page
 * - /cleanup-policies/create    → Create form
 * - /cleanup-policies/{name}    → Edit form
 */
type ViewMode = 'list' | 'create' | 'detail';

interface RouteState {
  viewMode: ViewMode;
  policyName: string | null;
}

/**
 * Parse the URL hash to determine the current route state
 */
function parseRoute(hash: string): RouteState {
  // Remove leading # and any query params (like ?debug)
  const cleanHash = hash.replace(/^#/, '').replace(/\?.*$/, '');
  const parts = cleanHash.split('/');

  const policiesIndex = parts.indexOf('cleanup-policies');
  if (policiesIndex === -1) {
    return { viewMode: 'list', policyName: null };
  }

  const pathAfterPolicies = parts.slice(policiesIndex + 1);

  // /cleanup-policies → list
  if (pathAfterPolicies.length === 0 || pathAfterPolicies[0] === '') {
    return { viewMode: 'list', policyName: null };
  }

  // /cleanup-policies/create → create form
  if (pathAfterPolicies[0] === 'create') {
    return { viewMode: 'create', policyName: null };
  }

  // /cleanup-policies/{name} → detail/edit
  return {
    viewMode: 'detail',
    policyName: decodeURIComponent(pathAfterPolicies[0]),
  };
}

/**
 * Navigate to a new route by updating the URL hash
 */
function navigateTo(path: string) {
  window.location.hash = path;
}

/**
 * CleanupPoliciesPage - Main Cleanup Policies management page for Preview UI
 *
 * Uses URL-based routing for testability:
 * - Direct URL access to any view
 * - Browser back/forward support
 * - Bookmarkable URLs
 *
 * URL patterns:
 * - #admin/repository/cleanup-policies           → List
 * - #admin/repository/cleanup-policies/create    → Create form
 * - #admin/repository/cleanup-policies/{name}    → Edit form
 */
export function CleanupPoliciesPage() {
  const [routeState, setRouteState] = useState<RouteState>(() => parseRoute(window.location.hash));
  const [policy, setPolicy] = useState<CleanupPolicy | null>(null);
  const [formatCriteria, setFormatCriteria] = useState<FormatCriteria[]>([]);
  const [refreshKey, setRefreshKey] = useState(0);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  // Toast notifications (app-level provider)
  const toast = useToast();

  const {
    loading,
    error,
    setError,
    fetchCleanupPolicy,
    fetchFormatCriteria,
    deleteCleanupPolicy,
  } = useCleanupPoliciesApi();

  const canCreate = ExtJS.checkPermission('astrum:settings:update');
  const canDelete = ExtJS.checkPermission('astrum:settings:update');

  // Listen for hash changes (browser back/forward, direct URL navigation)
  useEffect(() => {
    const handleHashChange = () => {
      const newState = parseRoute(window.location.hash);
      setRouteState(newState);
      setError(null);
    };

    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, [setError]);

  // Load format criteria on mount
  useEffect(() => {
    fetchFormatCriteria()
      .then(setFormatCriteria)
      .catch((err) => console.error('Failed to load format criteria:', err));
  }, [fetchFormatCriteria]);

  // Load policy details when editing
  useEffect(() => {
    if (routeState.viewMode === 'detail' && routeState.policyName) {
      fetchCleanupPolicy(routeState.policyName)
        .then((policyData) => {
          if (policyData) {
            setPolicy(policyData);
          } else {
            setError(CleanupPoliciesStrings.PAGE.CLEANUP_POLICY_NOT_FOUND);
            navigateTo(BASE_PATH);
          }
        })
        .catch((err) => {
          setError(err.message);
          navigateTo(BASE_PATH);
        });
    } else {
      setPolicy(null);
    }
  }, [routeState.viewMode, routeState.policyName, fetchCleanupPolicy, setError]);

  // Navigation handlers - update URL instead of local state
  const handleSelectPolicy = useCallback((name: string) => {
    navigateTo(`${BASE_PATH}/${encodeURIComponent(name)}`);
  }, []);

  const handleCreate = useCallback(() => {
    navigateTo(`${BASE_PATH}/create`);
  }, []);

  const handleBack = useCallback(() => {
    navigateTo(BASE_PATH);
  }, []);

  // Called by form after successful save - form's useCleanupPolicyForm handles API call + toast
  const handleSave = useCallback(async (_data: CleanupPolicyFormData) => {
    setRefreshKey((k) => k + 1);
  }, []);

  const handleDelete = useCallback(() => {
    setDeleteDialogOpen(true);
  }, []);

  const handleDeleteConfirm = useCallback(async () => {
    if (!policy) return;

    try {
      await deleteCleanupPolicy(policy.name);
      toast.success(CleanupPoliciesStrings.PAGE.CLEANUP_POLICY_DELETED_SUCCESSFULLY(policy.name));
      setRefreshKey((k) => k + 1);
      setRouteState({ viewMode: 'list', policyName: null });
      setDeleteDialogOpen(false);
    } catch (err) {
      const message = err instanceof Error ? err.message : CommonStrings.MESSAGES.OPERATION_FAILED;
      toast.error(message);
    }
  }, [policy, deleteCleanupPolicy, toast]);

  // Render header based on view mode
  const renderHeader = () => {
    if (routeState.viewMode === 'list') {
      return (
        <PageHeader
          title={CleanupPoliciesStrings.PAGE.CLEANUP_POLICIES}
          description={CleanupPoliciesStrings.PAGE.MANAGE_COMPONENT_REMOVAL_CONFIGURATION}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
            { label: CleanupPoliciesStrings.PAGE.CLEANUP_POLICIES }
          ]}
          actions={canCreate && (
            <SettingsButton
              variant="primary"
              onClick={handleCreate}
              icon={Plus}
              testId="create-policy-button"
              data-analytics-id="as-cleanup-policy-create"
            >
              {CleanupPoliciesStrings.PAGE.CREATE_CLEANUP_POLICY}
            </SettingsButton>
          )}
        />
      );
    }

    const title = routeState.viewMode === 'create'
      ? CleanupPoliciesStrings.PAGE.CREATE_CLEANUP_POLICY
      : policy
        ? CleanupPoliciesStrings.PAGE.EDIT(policy.name)
        : CleanupPoliciesStrings.PAGE.CLEANUP_POLICY_DETAILS;

    const description = policy && routeState.viewMode === 'detail'
      ? CleanupPoliciesStrings.PAGE.FORMAT(formatDisplayName(policy.format))
      : undefined;

    const lastBreadcrumb = routeState.viewMode === 'create'
      ? CommonStrings.ACTIONS.CREATE
      : policy?.name || CommonStrings.MESSAGES.LOADING;

    return (
      <PageHeader
        title={title}
        description={description}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
          { label: CleanupPoliciesStrings.PAGE.CLEANUP_POLICIES, onClick: handleBack },
          { label: lastBreadcrumb }
        ]}
      />
    );
  };

  // Show loading when in detail mode but policy not yet loaded
  if (routeState.viewMode === 'detail' && routeState.policyName && !policy) {
    return (
      <Box
        className="cleanup-policies-page"
        data-testid="cleanup-policies-page"
        data-view="loading"
      >
        {renderHeader()}
        <Box p="4">
          <LoadingState message={CleanupPoliciesStrings.PAGE.LOADING_CLEANUP_POLICY_DETAILS} />
        </Box>
      </Box>
    );
  }

  return (
    <Box
      className="cleanup-policies-page"
      data-testid="cleanup-policies-page"
      data-view={routeState.viewMode}
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      <Box mb="4">
        {renderHeader()}
      </Box>

      {/* Error Alert */}
      {error && (
        <Box className="cleanup-policies-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      <Box className="cleanup-policies-page__content">
        {routeState.viewMode === 'list' && (
          <CleanupPoliciesList
            key={refreshKey}
            onSelect={handleSelectPolicy}
            onCreate={handleCreate}
          />
        )}

        {routeState.viewMode === 'create' && (
          <CleanupPolicyForm
            isCreate={true}
            formatCriteria={formatCriteria}
            onSave={handleSave}
            onCancel={handleBack}
            loading={loading}
          />
        )}

        {routeState.viewMode === 'detail' && policy && (
          <CleanupPolicyForm
            policy={policy}
            isCreate={false}
            formatCriteria={formatCriteria}
            canDelete={canDelete}
            onSave={handleSave}
            onDelete={handleDelete}
            onCancel={handleBack}
            loading={loading}
          />
        )}
      </Box>

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        testId="delete-cleanup-policy-dialog"
        onOpenChange={setDeleteDialogOpen}
        title={CleanupPoliciesStrings.PAGE.DELETE_CLEANUP_POLICY}
        message={
          policy && policy.inUseCount > 0
            ? `${plural(CleanupPoliciesStrings.PAGE.USED_BY_REPOSITORIES, policy.inUseCount)} ${CleanupPoliciesStrings.PAGE.DELETE_WARNING}`
            : CleanupPoliciesStrings.PAGE.DELETE_WARNING
        }
        entityName={policy?.name}
        confirmLabel={CommonStrings.ACTIONS.DELETE}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="danger"
        onConfirm={handleDeleteConfirm}
      />
    </Box>
  );
}

export default CleanupPoliciesPage;
