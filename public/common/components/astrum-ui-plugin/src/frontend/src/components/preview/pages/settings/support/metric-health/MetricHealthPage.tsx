
import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { Box, Flex, Text, Grid } from '@radix-ui/themes';
import { Loader2, Download, RefreshCw, Info, ExternalLink, CheckCircle, XCircle } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsAlert, SettingsButton } from '../../../../shared/form';
import { useToast, PageHeader } from '../../../../shared';
import { MetricHealthList } from './MetricHealthList';
import { MetricHealthDetail } from './MetricHealthDetail';
import { useMetricHealthApi } from './useMetricHealthApi';
import { MetricHealthPageProps, HealthCheck, NodeInfo, formatCheckName } from './types';

import './MetricHealthPage.scss';
import MetricHealthStrings from '../../../../constants/pages/settings/support/MetricHealthStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * MetricHealthPage - Metric Health page for Preview UI
 *
 * Displays health check status for various system components.
 * Supports both single-node and clustered modes.
 */
export function MetricHealthPage({ className }: MetricHealthPageProps) {
  const {
    loading,
    error,
    setError,
    fetchMetricHealth,
    fetchClusterNodes,
    fetchNodeMetricHealth,
    downloadMetricHealth,
  } = useMetricHealthApi();

  const toast = useToast();

  const [checks, setChecks] = useState<HealthCheck[]>([]);
  const [selectedCheck, setSelectedCheck] = useState<string | null>(null);
  const [loadingInitial, setLoadingInitial] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  // Clustered mode state
  const [isClusteredMode, setIsClusteredMode] = useState(false);
  const [nodes, setNodes] = useState<NodeInfo[]>([]);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  const canRead = ExtJS.checkPermission('astrum:metrics:read');

  // Check if instance is clustered
  const isClustered = useCallback(() => {
    return ExtJS.state().getValue('astrum.datastore.clustered.enabled') === true;
  }, []);

  // Load health data on mount
  const loadData = useCallback(async () => {
    setLoadingInitial(true);
    setError(null);

    try {
      const clustered = isClustered();
      setIsClusteredMode(clustered);

      if (clustered) {
        // Clustered mode: fetch nodes list
        const clusterNodes = await fetchClusterNodes();
        setNodes(clusterNodes);
        // Don't auto-select a node - user must click to view details
        setSelectedNode(null);
      } else {
        // Single node mode: fetch health checks directly
        const healthChecks = await fetchMetricHealth();
        setChecks(healthChecks);

        // Select first unhealthy check, or first check if all healthy
        const unhealthyCheck = healthChecks.find((c) => !c.result.healthy);
        const firstCheck = healthChecks[0];
        setSelectedCheck(unhealthyCheck?.name || firstCheck?.name || null);
      }
    } catch (_err) {
      // Error is already set by the hook
    } finally {
      setLoadingInitial(false);
    }
  }, [fetchMetricHealth, fetchClusterNodes, isClustered, setError]);

  useEffect(() => {
    if (canRead) {
      loadData();
    } else {
      setLoadingInitial(false);
    }
  }, [canRead, loadData]);

  // Handle node selection in clustered mode
  const handleNodeSelect = useCallback(async (nodeId: string) => {
    setSelectedNode(nodeId);
    try {
      const healthChecks = await fetchNodeMetricHealth(nodeId);
      setChecks(healthChecks);

      // Select first unhealthy check, or first check if all healthy
      const unhealthyCheck = healthChecks.find((c) => !c.result.healthy);
      const firstCheck = healthChecks[0];
      setSelectedCheck(unhealthyCheck?.name || firstCheck?.name || null);
    } catch (_err) {
      // Error is set by the hook
    }
  }, [fetchNodeMetricHealth]);

  // Handle back to node list in clustered mode
  const handleBackToNodes = useCallback(() => {
    setSelectedNode(null);
    setChecks([]);
    setSelectedCheck(null);
  }, []);

  // Handle refresh
  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    try {
      if (isClusteredMode) {
        if (selectedNode) {
          // Refresh current node's health checks
          const healthChecks = await fetchNodeMetricHealth(selectedNode);
          setChecks(healthChecks);
        } else {
          // Refresh nodes list
          const clusterNodes = await fetchClusterNodes();
          setNodes(clusterNodes);
        }
      } else {
        const healthChecks = await fetchMetricHealth();
        setChecks(healthChecks);
      }
      toast.success(MetricHealthStrings.PAGE.HEALTH_CHECKS_REFRESHED);
    } catch (_err) {
      // Error is set by the hook
    } finally {
      setRefreshing(false);
    }
  }, [fetchMetricHealth, fetchClusterNodes, fetchNodeMetricHealth, isClusteredMode, selectedNode, toast.success]);

  // Handle download
  const handleDownload = useCallback(() => {
    if (checks.length === 0) return;
    const filename = selectedNode ? `metric-health-${selectedNode}.json` : 'metric-health.json';
    downloadMetricHealth(checks, filename);
    toast.success(MetricHealthStrings.PAGE.HEALTH_CHECK_DATA_DOWNLOADED);
  }, [checks, downloadMetricHealth, selectedNode, toast.success]);

  // Get selected check
  const selectedCheckData = useMemo(() => {
    return checks.find((c) => c.name === selectedCheck) || null;
  }, [checks, selectedCheck]);

  // Loading state
  if (loadingInitial) {
    return (
      <Box className={`metric-health-page ${className || ''}`.trim()} data-testid="metric-health-page">
        <Flex align="center" justify="center" className="metric-health-page__loading">
          <Loader2 size={24} className="metric-health-page__spinner" />
          <Text size="2">{MetricHealthStrings.PAGE.LOADING_HEALTH_CHECKS}</Text>
        </Flex>
      </Box>
    );
  }

  // Navigation helper for Settings breadcrumb
  const navigateToSettings = () => {
    window.location.hash = '#preview/admin/settings';
  };

  // No permission state
  if (!canRead) {
    const breadcrumbs = [
      { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
      { label: MetricHealthStrings.PAGE.METRIC_HEALTH },
    ];
    return (
      <Box className={`metric-health-page ${className || ''}`.trim()} data-testid="metric-health-page">
        <PageHeader
          title={MetricHealthStrings.PAGE.METRIC_HEALTH}
          description={MetricHealthStrings.PAGE.VIEW_SYSTEM_HEALTH_CHECKS_DIAGNOSTICS}
          breadcrumbs={breadcrumbs}
          className="metric-health-page__header"
        />

        <SettingsAlert type="warning">
          {MetricHealthStrings.PAGE.YOU_DO_NOT_HAVE_PERMISSION}
        </SettingsAlert>
      </Box>
    );
  }

  // Render node list for clustered mode when no node is selected
  const renderNodeList = () => (
    <Box className="metric-health-page__nodes">
      <Text size="2" weight="medium" className="metric-health-page__nodes-title">
        {MetricHealthStrings.PAGE.SELECT_NODE_VIEW_HEALTH_CHECKS}
      </Text>
      <Box className="metric-health-page__nodes-list">
        {nodes.length === 0 ? (
          <Text size="2" className="metric-health-page__empty">
            {MetricHealthStrings.PAGE.NO_CLUSTER_NODES_AVAILABLE}
          </Text>
        ) : (
          nodes.map((node) => (
            <button
              key={node.nodeId}
              type="button"
              className="metric-health-page__node-item"
              onClick={() => handleNodeSelect(node.nodeId)}
              data-testid={`node-item-${node.nodeId}`}
            >
              <Flex align="center" gap="3">
                {node.healthy !== undefined && (
                  node.healthy ? (
                    <CheckCircle size={20} className="metric-health-page__node-icon metric-health-page__node-icon--healthy" />
                  ) : (
                    <XCircle size={20} className="metric-health-page__node-icon metric-health-page__node-icon--unhealthy" />
                  )
                )}
                <Box>
                  <Text size="2" weight="medium" className="metric-health-page__node-name">
                    {node.hostname || node.nodeId}
                  </Text>
                  {node.message && (
                    <Text size="1" className="metric-health-page__node-message">
                      {node.message}
                    </Text>
                  )}
                </Box>
              </Flex>
            </button>
          ))
        )}
      </Box>
    </Box>
  );

  return (
    <Box className={`metric-health-page ${className || ''}`.trim()} data-testid="metric-health-page" data-loading={loading}>
      {/* Header */}
      {(() => {
        // The selected check is named in the breadcrumbs and in its detail card, not again in the description
        const description = isClusteredMode && selectedNode
          ? MetricHealthStrings.PAGE.HEALTH_CHECKS(nodes.find(n => n.nodeId === selectedNode)?.hostname || selectedNode)
          : MetricHealthStrings.PAGE.VIEW_SYSTEM_HEALTH_CHECKS_DIAGNOSTICS;

        const breadcrumbs = selectedCheck
          ? [
              { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
              { label: MetricHealthStrings.PAGE.METRIC_HEALTH, onClick: isClusteredMode && selectedNode ? handleBackToNodes : undefined },
              { label: formatCheckName(selectedCheck) },
            ]
          : [
              { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
              { label: MetricHealthStrings.PAGE.METRIC_HEALTH },
            ];

        const actions = (
          <Flex gap="2" className="metric-health-page__actions">
            <SettingsButton
              variant="ghost"
              onClick={handleRefresh}
              disabled={refreshing || loading}
              aria-label={CommonStrings.ACTIONS.REFRESH}
              icon={RefreshCw}
              className={refreshing ? 'metric-health-page__spinning' : ''}
              data-testid="refresh-button"
            >
              {CommonStrings.ACTIONS.REFRESH}
            </SettingsButton>
            {(!isClusteredMode || selectedNode) && (
              <SettingsButton
                variant="ghost"
                onClick={handleDownload}
                disabled={checks.length === 0 || loading}
                aria-label={CommonStrings.ACTIONS.DOWNLOAD}
                icon={Download}
                data-testid="download-button"
              >
                {CommonStrings.ACTIONS.DOWNLOAD}
              </SettingsButton>
            )}
          </Flex>
        );

        return (
          <PageHeader
            title={MetricHealthStrings.PAGE.METRIC_HEALTH}
            description={description}
            breadcrumbs={breadcrumbs}
            actions={actions}
            className="metric-health-page__header"
          />
        );
      })()}

      {/* Error Alert */}
      {error && (
        <Box className="metric-health-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      {isClusteredMode && !selectedNode ? (
        // Clustered mode: show node list
        renderNodeList()
      ) : checks.length === 0 ? (
        <Text size="2" className="metric-health-page__empty">
          {MetricHealthStrings.PAGE.NO_HEALTH_CHECKS_AVAILABLE}
        </Text>
      ) : (
        // Single node or node selected: show health checks
        <Grid columns="1fr 1fr" gap="4" className="metric-health-page__content">
          <MetricHealthList
            checks={checks}
            selectedCheck={selectedCheck}
            onSelectCheck={setSelectedCheck}
          />
          <MetricHealthDetail check={selectedCheckData} />
        </Grid>
      )}

      {/* Help Section */}
      <Box className="metric-health-page__help">
        <Flex align="center" gap="2" className="metric-health-page__help-header">
          <Info size={16} />
          <Text size="2" weight="medium">{MetricHealthStrings.PAGE.ABOUT_STATUS}</Text>
        </Flex>
        <Text as="p" size="2" className="metric-health-page__help-text">
          {MetricHealthStrings.PAGE.PAGE_DISPLAYS_STATUS_VARIOUS_HEALTH}
        </Text>
        <Text as="p" size="2" className="metric-health-page__help-text">
          {MetricHealthStrings.PAGE.SEE_DOCUMENTATION(
            <a
              href="http://links.globalart.dev/products/astrum/docs/metrics"
              target="_blank"
              rel="noopener noreferrer"
              className="metric-health-page__help-link"
            >
              {MetricHealthStrings.PAGE.DOCUMENTATION_LINK}
              <ExternalLink size={12} />
            </a>
          )}
        </Text>
      </Box>
    </Box>
  );
}

export default MetricHealthPage;


