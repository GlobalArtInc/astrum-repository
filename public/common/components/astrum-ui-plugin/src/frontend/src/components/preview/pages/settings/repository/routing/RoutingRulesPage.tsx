

import React, { useState, useEffect, useCallback } from 'react';
import { Box, Flex } from '@radix-ui/themes';
import { Eye, Plus } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { useToast, LoadingState, PageHeader } from '../../../../shared';
import { SettingsButton, SettingsAlert, } from '../../../../shared/form';
import { DeleteConfirmationModal } from '../../../../shared/modals/DeleteConfirmationModal';
import { RoutingRulesList } from './RoutingRulesList';
import { RoutingRuleForm } from './RoutingRuleForm';
import { RoutingRulePreview } from './RoutingRulePreview';
import { useRoutingRulesApi } from './useRoutingRulesApi';
import { RoutingRule, RoutingRuleFormData } from './types';

import './RoutingRulesPage.scss';
import RoutingRulesStrings from '../../../../constants/pages/settings/repository/RoutingRulesStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

// Base path for routing rules URLs
const BASE_PATH = 'preview/admin/repository/routing-rules';

/**
 * URL-based routing patterns:
 * - /routing-rules           → List page
 * - /routing-rules/create    → Create form
 * - /routing-rules/preview   → Global preview
 * - /routing-rules/{name}    → Edit form
 */
type ViewMode = 'list' | 'create' | 'detail' | 'preview';

interface RouteState {
  viewMode: ViewMode;
  ruleName: string | null;
}

/**
 * Parse the URL hash to determine the current route state
 */
function parseRoute(hash: string): RouteState {
  // Remove leading # and any query params (like ?debug)
  const cleanHash = hash.replace(/^#/, '').replace(/\?.*$/, '');
  const parts = cleanHash.split('/');

  const rulesIndex = parts.indexOf('routing-rules');
  if (rulesIndex === -1) {
    return { viewMode: 'list', ruleName: null };
  }

  const pathAfterRules = parts.slice(rulesIndex + 1);

  // /routing-rules → list
  if (pathAfterRules.length === 0 || pathAfterRules[0] === '') {
    return { viewMode: 'list', ruleName: null };
  }

  // /routing-rules/create → create form
  if (pathAfterRules[0] === 'create') {
    return { viewMode: 'create', ruleName: null };
  }

  // /routing-rules/preview → global preview
  if (pathAfterRules[0] === 'preview') {
    return { viewMode: 'preview', ruleName: null };
  }

  // /routing-rules/{name} → detail/edit
  return {
    viewMode: 'detail',
    ruleName: decodeURIComponent(pathAfterRules[0]),
  };
}

/**
 * Navigate to a new route by updating the URL hash
 */
function navigateTo(path: string) {
  window.location.hash = path;
}

/**
 * RoutingRulesPage - Main Routing Rules management page for Preview UI
 *
 * Uses URL-based routing for testability:
 * - Direct URL access to any view
 * - Browser back/forward support
 * - Bookmarkable URLs
 *
 * URL patterns:
 * - #preview/admin/repository/routing-rules           → List
 * - #preview/admin/repository/routing-rules/create    → Create form
 * - #preview/admin/repository/routing-rules/preview   → Global preview
 * - #preview/admin/repository/routing-rules/{name}    → Edit form
 */
export function RoutingRulesPage() {
  const [routeState, setRouteState] = useState<RouteState>(() => parseRoute(window.location.hash));
  const [rule, setRule] = useState<RoutingRule | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  // Toast notifications (app-level provider)
  const toast = useToast();

  const {
    loading,
    error,
    setError,
    fetchRoutingRule,
    deleteRoutingRule,
  } = useRoutingRulesApi();

  const canCreate = ExtJS.checkPermission('astrum:*');
  const canDelete = ExtJS.checkPermission('astrum:*');

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

  // Load rule details when editing
  useEffect(() => {
    if (routeState.viewMode === 'detail' && routeState.ruleName) {
      setRule(null); // Clear stale rule before fetching to avoid showing wrong data
      fetchRoutingRule(routeState.ruleName)
        .then((ruleData) => {
          if (ruleData) {
            setRule(ruleData);
          } else {
            setError(RoutingRulesStrings.PAGE.ROUTING_RULE_NOT_FOUND);
            navigateTo(BASE_PATH);
          }
        })
        .catch((err) => {
          setError(err.message);
          navigateTo(BASE_PATH);
        });
    } else {
      setRule(null);
    }
  }, [routeState.viewMode, routeState.ruleName, fetchRoutingRule, setError]);

  // Navigation handlers - update URL instead of local state
  const handleSelectRule = useCallback((name: string) => {
    navigateTo(`${BASE_PATH}/${encodeURIComponent(name)}`);
  }, []);

  const handleCreate = useCallback(() => {
    navigateTo(`${BASE_PATH}/create`);
  }, []);

  const handlePreview = useCallback(() => {
    navigateTo(`${BASE_PATH}/preview`);
  }, []);

  const handleBack = useCallback(() => {
    navigateTo(BASE_PATH);
  }, []);

  // Called by form after successful save - form's useRoutingRulesForm handles API call + toast
  const handleSave = useCallback((_data: RoutingRuleFormData): Promise<void> => {
    setRefreshKey((k) => k + 1);
    return Promise.resolve();
  }, []);

  const handleDeleteClick = useCallback(() => {
    setDeleteDialogOpen(true);
  }, []);

  const handleDeleteConfirm = useCallback(async () => {
    if (!rule) return;

    setIsDeleting(true);
    try {
      await deleteRoutingRule(rule.name);
      toast.success(RoutingRulesStrings.PAGE.ROUTING_RULE_DELETED_SUCCESSFULLY(rule.name));
      setRefreshKey((k) => k + 1);
      setDeleteDialogOpen(false);
      navigateTo(BASE_PATH);
    } catch (err) {
      const message = err instanceof Error ? err.message : RoutingRulesStrings.PAGE.FAILED_DELETE_ROUTING_RULE;
      setError(message);
      toast.error(message);
    } finally {
      setIsDeleting(false);
    }
  }, [rule, deleteRoutingRule, toast, setError]);

  // Render header based on view mode
  const renderHeader = () => {
    if (routeState.viewMode === 'list') {
      return (
        <PageHeader
          title={RoutingRulesStrings.PAGE.ROUTING_RULES}
          description={RoutingRulesStrings.PAGE.CONTROL_WHICH_REQUESTS_ALLOWED_BLOCKED}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: RoutingRulesStrings.PAGE.ROUTING_RULES }
          ]}
          actions={
            <Flex gap="2">
              <SettingsButton
                variant="secondary"
                onClick={handlePreview}
                icon={Eye}
                testId="preview-button"
              >
                {RoutingRulesStrings.PAGE.PREVIEW}
              </SettingsButton>
              {canCreate && (
                <SettingsButton
                  variant="primary"
                  onClick={handleCreate}
                  icon={Plus}
                  testId="create-rule-button"
                >
                  {RoutingRulesStrings.PAGE.CREATE_RULE}
                </SettingsButton>
              )}
            </Flex>
          }
        />
      );
    }

    if (routeState.viewMode === 'preview') {
      return (
        <PageHeader
          title={RoutingRulesStrings.PAGE.GLOBAL_ROUTING_PREVIEW}
          description={RoutingRulesStrings.PAGE.TEST_HOW_ROUTING_RULES_AFFECT}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: RoutingRulesStrings.PAGE.ROUTING_RULES, onClick: handleBack },
            { label: RoutingRulesStrings.PAGE.PREVIEW }
          ]}
        />
      );
    }

    const title = routeState.viewMode === 'create'
      ? RoutingRulesStrings.PAGE.CREATE_ROUTING_RULE
      : rule
        ? RoutingRulesStrings.PAGE.EDIT(rule.name)
        : RoutingRulesStrings.PAGE.ROUTING_RULE_DETAILS;

    const description = rule && routeState.viewMode === 'detail'
      ? `${RoutingRulesStrings.PAGE.DETAIL_DESCRIPTION(RoutingRulesStrings.PAGE.MODE_NAMES[rule.mode as 'ALLOW' | 'BLOCK'] ?? rule.mode, rule.matchers.length)}${(rule.assignedRepositoryCount ?? 0) > 0 ? ` • ${plural(RoutingRulesStrings.PAGE.ASSIGNED_TO_REPOSITORIES, rule.assignedRepositoryCount ?? 0)}` : ''}`
      : undefined;

    const lastBreadcrumb = routeState.viewMode === 'create'
      ? CommonStrings.ACTIONS.CREATE
      : rule?.name || CommonStrings.MESSAGES.LOADING;

    return (
      <PageHeader
        title={title}
        description={description}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
          { label: RoutingRulesStrings.PAGE.ROUTING_RULES, onClick: handleBack },
          { label: lastBreadcrumb }
        ]}
      />
    );
  };

  // Show loading when in detail mode but rule not yet loaded
  if (routeState.viewMode === 'detail' && routeState.ruleName && !rule) {
    return (
      <Box
        className="routing-rules-page"
        data-testid="routing-rules-page"
        data-view="loading"
        px={{ initial: '4', md: '6', lg: '6' }}
        py={{ initial: '4', md: '5', lg: '6' }}
      >
        <Box mb="4">
          {renderHeader()}
        </Box>
        <Box p="4" className="routing-rules-page__content">
          <LoadingState message={RoutingRulesStrings.PAGE.LOADING_ROUTING_RULE_DETAILS} />
        </Box>
      </Box>
    );
  }

  return (
    <Box
      className="routing-rules-page"
      data-testid="routing-rules-page"
      data-view={routeState.viewMode}
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      <Box mb="4">
        {renderHeader()}
      </Box>

      {/* Error Alert */}
      {error && (
        <Box className="routing-rules-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      <Box className="routing-rules-page__content">
        {routeState.viewMode === 'list' && (
          <RoutingRulesList
            key={refreshKey}
            onSelect={handleSelectRule}
            onCreate={handleCreate}
            onPreview={handlePreview}
          />
        )}

        {routeState.viewMode === 'create' && (
          <RoutingRuleForm
            isCreate={true}
            onSave={handleSave}
            onCancel={handleBack}
            loading={loading}
          />
        )}

        {routeState.viewMode === 'detail' && rule && (
          <RoutingRuleForm
            rule={rule}
            isCreate={false}
            onSave={handleSave}
            onCancel={handleBack}
            onDelete={canDelete ? handleDeleteClick : undefined}
            loading={loading}
          />
        )}

        {routeState.viewMode === 'preview' && (
          <RoutingRulePreview onClose={handleBack} />
        )}
      </Box>

      {/* Delete Confirmation Modal */}
      <DeleteConfirmationModal
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        onConfirm={handleDeleteConfirm}
        entityName={rule?.name || ''}
        entityType="routing rule"
        loading={isDeleting}
      />
    </Box>
  );
}

export default RoutingRulesPage;
