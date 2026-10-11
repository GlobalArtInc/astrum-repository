

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import {
  Box,
  DropdownMenu,
  Flex,
  Grid,
  Heading,
  Spinner,
  Table,
  Text,
  TextField,
  Tooltip,
  IconButton,
  Select,
} from '@radix-ui/themes';
import {
  ArrowUpDown,
  Copy,
  Database,
  Plus,
  Search,
  X,
  MoreHorizontal,
} from 'lucide-react';
import { DeleteConfirmationModal, TablePagination } from '../../../../shared';
import { ConfirmDialog } from '../../../../shared/ConfirmDialog';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { restClient, ENDPOINTS } from '../../../../../../interface/api';

import {
  EmptyState,
  ErrorState,
  HelpSection,
  SortableTableHeader,
  useToast,
} from '../../../../shared';

import { BrowseFilterSidebar } from '../../../browse/repository-list/BrowseFilterSidebar';

import { useRepositoriesApi } from './useRepositoriesApi';
import {
  Repository,
  RepositoriesListProps,
  SortDirection,
  RepositorySortField,
  FORMAT_LABELS,
  TYPE_LABELS,
  HealthCheckStatus,
} from './types';
import { HealthCheckCell } from '../../../../shared/security/HealthCheckCell';
import { FirewallCell } from '../../../../shared/security/FirewallCell';

import './RepositoriesList.scss';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface FirewallStatusData {
  repositoryName: string;
  affectedComponentCount: number;
  criticalComponentCount: number;
  severeComponentCount: number;
  moderateComponentCount: number;
  quarantinedComponentCount: number;
  reportUrl?: string;
  message?: string | null;
  errorMessage?: string | null;
}

function isIqServerEnabled(): boolean {
  try { return ExtJS.state().getValue('clm')?.enabled ?? false; }
  catch { return false; }
}

function hasFirewall(): boolean {
  try { return ExtJS.state().getValue('clm')?.hasFirewall ?? false; }
  catch { return false; }
}

function canUpdateHealthCheck(): boolean {
  try { return ExtJS.checkPermission('astrum:healthcheck:update'); }
  catch { return false; }
}

function canReadFirewallStatus(): boolean {
  try { return ExtJS.checkPermission('astrum:iq-violation-summary:read'); }
  catch { return false; }
}

/**
 * Health Check (RHC) is a lower-tier feature that is superseded by the IQ Server's Firewall product. When the
 * customer already has Firewall configured (IQ Server is enabled AND the persisted {@code clm.hasFirewall} flag
 * is {@code true}), the RHC column is redundant — the Firewall Report column shows richer data — so we hide it,
 * matching the classic UI behaviour (AS.Conditions.hasNoFirewall in ExtJS). Requires the healthcheck:update
 * permission on top of the tier gate (ASTRUM-53278).
 */
function shouldShowHealthCheckColumn(): boolean {
  return canUpdateHealthCheck() && !(isIqServerEnabled() && hasFirewall());
}

/**
 * Format file size in bytes to human-readable format
 */
function formatFileSize(bytes?: number): string {
  if (bytes == null || bytes === 0) {
    return '0 B';
  }
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const size = bytes / Math.pow(k, i);
  return `${size.toFixed(2)} ${units[i]}`;
}

const DEFAULT_PAGE_SIZE = 40;

// Pending confirmation state for operational row actions (rebuild index,
// invalidate cache, toggle online). Mirrors the pattern in RepositoriesPage:
// the menu item stages a pending action; ConfirmDialog renders the prompt
// and triggers execution on confirm.
type PendingRowAction =
  | { kind: 'rebuild-index'; repository: Repository }
  | { kind: 'invalidate-cache'; repository: Repository }
  | { kind: 'toggle-online'; repository: Repository; nextOnline: boolean };

/**
 * RepositoriesList - Displays repositories using shared components
 */
export function RepositoriesList({ onSelect, onCreate, onDelete }: RepositoriesListProps) {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [healthStatus, setHealthStatus] = useState<Record<string, HealthCheckStatus>>({});
  const [filter, setFilter] = useState('');
  const [typeFilter, setTypeFilter] = useState<string[]>([]);
  const [formatFilter, setFormatFilter] = useState<string[]>([]);
  const [sortField, setSortField] = useState<RepositorySortField>('name');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [loadingRepos, setLoadingRepos] = useState(true);
  const [analyzingRepos, setAnalyzingRepos] = useState<Set<string>>(new Set());
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [repoToDelete, setRepoToDelete] = useState<Repository | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [firewallStatus, setFirewallStatus] = useState<Record<string, FirewallStatusData>>({});
  const [firewallLoaded, setFirewallLoaded] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);

  const {
    error,
    setError,
    fetchRepositories,
    fetchRepository,
    deleteRepository,
    invalidateCache,
    rebuildIndex,
    setRepositoryOnline,
  } = useRepositoriesApi();

  const [pendingRowAction, setPendingRowAction] = useState<PendingRowAction | null>(null);
  const [isExecutingRowAction, setIsExecutingRowAction] = useState(false);

  const [showHealthCheckColumn, setShowHealthCheckColumn] = useState(() => shouldShowHealthCheckColumn());
  const [showFirewallColumn, setShowFirewallColumn] = useState(() => isIqServerEnabled() && canReadFirewallStatus());

  // Toast notifications
  const toast = useToast();

  // Check permissions
  const canEdit = ExtJS.checkPermission('astrum:repository-admin:*:*:edit');
  const canDelete = ExtJS.checkPermission('astrum:repository-admin:*:*:delete');

  const fetchHealthCheck = useCallback(async () => {
    try {
      const data = await restClient.get<Array<HealthCheckStatus & { repositoryName: string }>>(ENDPOINTS.HEALTH_CHECK);
      return (data || []).reduce<Record<string, HealthCheckStatus>>((acc, item) => {
        acc[item.repositoryName] = item;
        return acc;
      }, {});
    } catch {
      return {};
    }
  }, []);

  const fetchFirewallStatus = useCallback(async () => {
    try {
      const data = await restClient.get<FirewallStatusData[]>(ENDPOINTS.FIREWALL_STATUS_SUMMARY);
      return (data || []).reduce<Record<string, FirewallStatusData>>((acc, item) => {
        acc[item.repositoryName] = item;
        return acc;
      }, {});
    } catch {
      return {};
    }
  }, []);

  // Load repositories on mount
  useEffect(() => {
    const loadData = async () => {
      setLoadingRepos(true);
      try {
        const repos = await fetchRepositories();
        setRepositories(repos);
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : RepositoriesPageStrings.LIST.FAILED_LOAD_REPOSITORIES;
        setError(message);
      } finally {
        setLoadingRepos(false);
      }

      // Re-check permissions and IQ/Firewall state after ExtJS state may have loaded. Health Check must be
      // hidden when Firewall is configured (ASTRUM-53278), so re-evaluate the full gate — do not just
      // latch to true — otherwise a late-arriving clm state that flips hasFirewall on won't hide the column.
      setShowHealthCheckColumn(shouldShowHealthCheckColumn());
      const firewallEnabled = isIqServerEnabled() && canReadFirewallStatus();
      if (firewallEnabled) setShowFirewallColumn(true);

      const [health, firewall] = await Promise.all([
        fetchHealthCheck(),
        fetchFirewallStatus(),
      ]);
      setHealthStatus(health);
      setFirewallStatus(firewall);
      setFirewallLoaded(true);

      // Only surface the Health Check column when the tier gate allows it; the presence of health data
      // alone is not sufficient (a Firewall-licensed customer may still have leftover RHC records).
      if (Object.keys(health).length > 0 && shouldShowHealthCheckColumn()) {
        setShowHealthCheckColumn(true);
      }
      if (Object.keys(firewall).length > 0) setShowFirewallColumn(true);
    };

    loadData();
  }, [fetchRepositories, fetchHealthCheck, fetchFirewallStatus, setError]);

  // Get unique formats and types for filters with counts
  const { formatOptions, typeOptions } = useMemo(() => {
    const formatCounts = new Map<string, number>();
    const typeCounts = new Map<string, number>();
    
    repositories.forEach((repo) => {
      formatCounts.set(repo.format, (formatCounts.get(repo.format) || 0) + 1);
      typeCounts.set(repo.type, (typeCounts.get(repo.type) || 0) + 1);
    });
    
    return {
      formatOptions: Array.from(formatCounts.entries())
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([value, count]) => ({
          value,
          label: FORMAT_LABELS[value] || value,
          count,
        })),
      typeOptions: Array.from(typeCounts.entries())
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([value, count]) => ({
          value,
          label: TYPE_LABELS[value as keyof typeof TYPE_LABELS] || value,
          count,
        })),
    };
  }, [repositories]);

  // Filter repositories
  const filteredRepositories = useMemo(() => {
    return repositories.filter((repo) => {
      // Text filter
      if (filter) {
        const searchLower = filter.toLowerCase();
        const matchesName = repo.name?.toLowerCase().includes(searchLower);
        const matchesFormat = repo.format?.toLowerCase().includes(searchLower);
        if (!matchesName && !matchesFormat) {
          return false;
        }
      }
      
      // Type filter (array now)
      if (typeFilter.length > 0 && !typeFilter.includes(repo.type)) {
        return false;
      }
      
      // Format filter (array now)
      if (formatFilter.length > 0 && !formatFilter.includes(repo.format)) {
        return false;
      }
      
      return true;
    });
  }, [repositories, filter, typeFilter, formatFilter]);

  // Sort repositories
  const sortedRepositories = useMemo(() => {
    if (!sortDirection) return filteredRepositories;

    return [...filteredRepositories].sort((a, b) => {
      let aVal: string | boolean = '';
      let bVal: string | boolean = '';

      switch (sortField) {
        case 'name':
          aVal = a.name || '';
          bVal = b.name || '';
          break;
        case 'type':
          aVal = a.type || '';
          bVal = b.type || '';
          break;
        case 'format':
          aVal = a.format || '';
          bVal = b.format || '';
          break;
        case 'status':
          aVal = a.status?.online ?? a.online ?? true;
          bVal = b.status?.online ?? b.online ?? true;
          if (aVal !== bVal) {
            return sortDirection === 'asc'
              ? (aVal ? -1 : 1)
              : (aVal ? 1 : -1);
          }
          aVal = a.name || '';
          bVal = b.name || '';
          break;
      }

      const comparison = String(aVal).toLowerCase().localeCompare(String(bVal).toLowerCase());
      return sortDirection === 'asc' ? comparison : -comparison;
    });
  }, [filteredRepositories, sortField, sortDirection]);

  const handleSort = useCallback((key: string, direction: SortDirection) => {
    setSortField(key as RepositorySortField);
    setSortDirection(direction);
    setCurrentPage(1);
  }, []);

  // Handle sort changes from BrowseFilterSidebar
  const handleSortChange = useCallback((field: string, direction: 'asc' | 'desc') => {
    setSortField(field as RepositorySortField);
    setSortDirection(direction);
    setCurrentPage(1);
  }, []);

  // Handle reset filters
  const handleResetFilters = useCallback(() => {
    setFilter('');
    setTypeFilter([]);
    setFormatFilter([]);
    setSortField('name');
    setSortDirection('asc');
    setCurrentPage(1);
  }, []);

  const handleClearFilters = useCallback(() => {
    setFilter('');
    setTypeFilter([]);
    setFormatFilter([]);
    setCurrentPage(1);
  }, []);

  const handleRetry = useCallback(async () => {
    setError(null);
    setLoadingRepos(true);
    setFirewallLoaded(false);
    try {
      const repos = await fetchRepositories();
      setRepositories(repos);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : RepositoriesPageStrings.LIST.FAILED_LOAD_REPOSITORIES;
      setError(message);
    } finally {
      setLoadingRepos(false);
    }
    // Mirrors the mount-effect logic — re-evaluate the tier gate rather than latching, so retries pick up
    // any change in the IQ/Firewall configuration since the last attempt (ASTRUM-53278).
    setShowHealthCheckColumn(shouldShowHealthCheckColumn());
    const firewallEnabled = isIqServerEnabled() && canReadFirewallStatus();
    if (firewallEnabled) setShowFirewallColumn(true);

    const [health, firewall] = await Promise.all([
      fetchHealthCheck(),
      fetchFirewallStatus(),
    ]);
    setHealthStatus(health);
    setFirewallStatus(firewall);
    setFirewallLoaded(true);
    if (Object.keys(health).length > 0 && shouldShowHealthCheckColumn()) {
      setShowHealthCheckColumn(true);
    }
    if (Object.keys(firewall).length > 0) setShowFirewallColumn(true);
  }, [fetchRepositories, fetchHealthCheck, fetchFirewallStatus, setError]);

  const handleAnalyze = useCallback(async (repositoryName: string) => {
    setAnalyzingRepos((prev) => new Set(prev).add(repositoryName));
    try {
      await restClient.post(ENDPOINTS.HEALTH_CHECK_ANALYZE(repositoryName));
      const health = await fetchHealthCheck();
      setHealthStatus(health);
      toast.success(RepositoriesPageStrings.LIST.HEALTH_CHECK_ENABLED(repositoryName));
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : RepositoriesPageStrings.LIST.FAILED_ENABLE_HEALTH_CHECK;
      toast.error(RepositoriesPageStrings.LIST.FAILED_START_ANALYSIS(repositoryName, message));
    } finally {
      setAnalyzingRepos((prev) => {
        const next = new Set(prev);
        next.delete(repositoryName);
        return next;
      });
    }
  }, [fetchHealthCheck, toast]);

  // Handle delete confirmation
  const handleDeleteConfirm = useCallback(async () => {
    if (!repoToDelete) return;

    setIsDeleting(true);
    try {
      if (onDelete) {
        await onDelete(repoToDelete.name);
      } else {
        await deleteRepository(repoToDelete.name);
      }
      // Remove from local state
      setRepositories((prev) => prev.filter((r) => r.name !== repoToDelete.name));
      toast.success(RepositoriesPageStrings.LIST.REPOSITORY_DELETED_SUCCESSFULLY(repoToDelete.name));
      setDeleteModalOpen(false);
      setRepoToDelete(null);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : RepositoriesPageStrings.LIST.FAILED_DELETE_REPOSITORY;
      toast.error(message);
    } finally {
      setIsDeleting(false);
    }
  }, [repoToDelete, onDelete, deleteRepository, toast]);

  // Row-action stagers — clicking a menu item opens the ConfirmDialog; the
  // actual REST call fires from handleConfirmRowAction.
  const stageRebuildIndex = useCallback((repo: Repository) => {
    setPendingRowAction({ kind: 'rebuild-index', repository: repo });
  }, []);

  const stageInvalidateCache = useCallback((repo: Repository) => {
    setPendingRowAction({ kind: 'invalidate-cache', repository: repo });
  }, []);

  const stageToggleOnline = useCallback((repo: Repository) => {
    const currentOnline = repo.status?.online ?? repo.online ?? true;
    setPendingRowAction({ kind: 'toggle-online', repository: repo, nextOnline: !currentOnline });
  }, []);

  const cancelPendingRowAction = useCallback(() => {
    if (isExecutingRowAction) return;
    setPendingRowAction(null);
  }, [isExecutingRowAction]);

  const handleConfirmRowAction = useCallback(async () => {
    if (!pendingRowAction) return;
    const { repository: repo } = pendingRowAction;
    const repoName = repo.name;
    setIsExecutingRowAction(true);
    try {
      if (pendingRowAction.kind === 'rebuild-index') {
        await rebuildIndex(repoName);
        toast.success(RepositoriesPageStrings.LIST.REPOSITORY_INDEX_REBUILD_STARTED(repoName));
      } else if (pendingRowAction.kind === 'invalidate-cache') {
        await invalidateCache(repoName);
        toast.success(RepositoriesPageStrings.LIST.REPOSITORY_CACHES_INVALIDATED(repoName));
      } else {
        const nextOnline = pendingRowAction.nextOnline;
        await setRepositoryOnline(repoName, repo.format, repo.type, nextOnline);
        toast.success(
          nextOnline
            ? RepositoriesPageStrings.LIST.REPOSITORY_NOW_ONLINE(repoName)
            : RepositoriesPageStrings.LIST.REPOSITORY_NOW_OFFLINE(repoName)
        );
        // Refetch just this repository so the Status column reflects the
        // saved state. Failure to refetch would leave the row labelled with
        // the previous state until the list is reloaded.
        const refreshed = await fetchRepository(repoName);
        if (refreshed) {
          setRepositories((prev) => prev.map((r) => (r.name === repoName ? { ...r, ...refreshed } : r)));
        }
      }
      setPendingRowAction(null);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : CommonStrings.MESSAGES.UNKNOWN_ERROR;
      toast.error(RepositoriesPageStrings.LIST.ACTION_FAILED, message);
    } finally {
      setIsExecutingRowAction(false);
    }
  }, [pendingRowAction, rebuildIndex, invalidateCache, setRepositoryOnline, fetchRepository, toast]);

  const rowActionDialogProps = useMemo(() => {
    if (!pendingRowAction) {
      return { title: '', message: '', confirmLabel: CommonStrings.ACTIONS.CONFIRM, variant: 'warning' as const };
    }
    const name = pendingRowAction.repository.name;
    if (pendingRowAction.kind === 'rebuild-index') {
      return {
        title: RepositoriesPageStrings.LIST.REBUILD_REPOSITORY_INDEX,
        message: RepositoriesPageStrings.LIST.REBUILD_SEARCH_INDEX_REPOSITORY_MAY(name),
        confirmLabel: RepositoriesPageStrings.LIST.REBUILD_INDEX,
        variant: 'warning' as const,
      };
    }
    if (pendingRowAction.kind === 'invalidate-cache') {
      return {
        title: RepositoriesPageStrings.LIST.INVALIDATE_REPOSITORY_CACHE,
        message: RepositoriesPageStrings.LIST.INVALIDATE_CACHED_METADATA_CONTENT_REPOSITORY(name),
        confirmLabel: RepositoriesPageStrings.LIST.INVALIDATE_CACHE,
        variant: 'warning' as const,
      };
    }
    const isGoingOffline = !pendingRowAction.nextOnline;
    return {
      title: isGoingOffline ? RepositoriesPageStrings.LIST.DISABLE_SYSTEM_STATUS : RepositoriesPageStrings.LIST.ENABLE_SYSTEM_STATUS,
      message: isGoingOffline
        ? RepositoriesPageStrings.LIST.TAKE_REPOSITORY_OFFLINE_CLIENTS_NO(name)
        : RepositoriesPageStrings.LIST.BRING_REPOSITORY_ONLINE_CLIENTS_ABLE(name),
      confirmLabel: isGoingOffline ? RepositoriesPageStrings.LIST.TAKE_OFFLINE : RepositoriesPageStrings.LIST.BRING_ONLINE,
      variant: 'warning' as const,
    };
  }, [pendingRowAction]);

  // Pagination
  const totalPages = Math.ceil(sortedRepositories.length / pageSize);
  const paginatedRepositories = useMemo(() => {
    const start = (currentPage - 1) * pageSize;
    const end = start + pageSize;
    return sortedRepositories.slice(start, end);
  }, [sortedRepositories, currentPage, pageSize]);

  const handlePageChange = useCallback((page: number) => {
    setCurrentPage(page);
  }, []);

  const handlePageSizeChange = useCallback((newPageSize: number) => {
    setPageSize(newPageSize);
    setCurrentPage(1); // Reset to first page
  }, []);

  // Handle row click
  const handleRowClick = (name: string) => () => {
    onSelect(name);
  };

  // Handle row keyboard navigation
  const handleRowKeyDown = (name: string) => (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onSelect(name);
    }
  };

  // Handle copy URL with toast
  const handleCopyUrlButton = (url: string) => (e: React.MouseEvent) => {
    e.stopPropagation();
    // Ensure URL has trailing slash (ASTRUM-53999)
    const urlWithSlash = url.endsWith('/') ? url : `${url}/`;
    navigator.clipboard.writeText(urlWithSlash)
      .then(() => {
        toast.success(RepositoriesPageStrings.LIST.URL_COPIED_CLIPBOARD);
      })
      .catch(() => {
        toast.error(RepositoriesPageStrings.LIST.FAILED_COPY_URL_CLIPBOARD);
      });
  };

  // Calculate column count for empty state (Name, Size, Type, Format, BlobStore, Status, URL, Actions)
  const colCount = 8 + (showHealthCheckColumn ? 1 : 0) + (showFirewallColumn ? 1 : 0);

  // Empty state for when no repositories exist
  const emptyState = useMemo(() => {
    const hasFilters = filter || typeFilter.length > 0 || formatFilter.length > 0;
    
    if (hasFilters) {
      return (
        <EmptyState
          icon={Database}
          title={RepositoriesPageStrings.LIST.NO_MATCHING_REPOSITORIES}
          description={RepositoriesPageStrings.LIST.NO_REPOSITORIES_MATCH_CURRENT_FILTERS}
          action={{
            label: CommonStrings.ACTIONS.CLEAR_FILTERS,
            onClick: handleClearFilters,
          }}
        />
      );
    }

    return (
      <EmptyState
        icon={Database}
        title={RepositoriesPageStrings.LIST.NO_REPOSITORIES}
        description={RepositoriesPageStrings.LIST.CREATE_FIRST_REPOSITORY_START_STORING}
        action={onCreate ? {
          label: RepositoriesPageStrings.LIST.CREATE_REPOSITORY,
          onClick: onCreate,
          icon: Plus,
        } : undefined}
        secondaryAction={{
          label: RepositoriesPageStrings.LIST.LEARN_MORE_ABOUT_REPOSITORIES,
          href: 'https://help.globalart.dev/en/repository-administration.html',
        }}
        tip={RepositoriesPageStrings.LIST.START_PROXY_REPOSITORY_CACHE_ARTIFACTS}
      />
    );
  }, [filter, typeFilter, formatFilter, handleClearFilters, onCreate]);

  const displayTotal = sortedRepositories.length.toLocaleString();

  const filterBarContent = (
    <BrowseFilterSidebar
      formatOptions={formatOptions}
      typeOptions={typeOptions}
      statusOptions={[]}
      selectedFormats={formatFilter}
      selectedTypes={typeFilter}
      selectedStatuses={[]}
      selectedProtection={[]}
      selectedHealthCheck={[]}
      sortField={sortField}
      sortDirection={sortDirection || 'asc'}
      onFormatsChange={(formats) => { setFormatFilter(formats); setCurrentPage(1); }}
      onTypesChange={(types) => { setTypeFilter(types); setCurrentPage(1); }}
      onStatusesChange={() => {}}
      onSortChange={handleSortChange}
      onResetFilters={handleResetFilters}
      disabled={loadingRepos}
    />
  );

  const renderSortDropdown = () => (
    <Select.Root
      value={sortField}
      onValueChange={(field) => handleSortChange(field, sortDirection || 'asc')}
      size="2"
    >
      <Select.Trigger style={{ width: 180, flexShrink: 0 }}>
        <Flex align="center" gap="2">
          <ArrowUpDown size={14} aria-hidden />
          <Text size="2">{RepositoriesPageStrings.LIST.SORT}</Text>
          <Text size="2">
            {sortField === 'name' ? CommonStrings.LABELS.NAME : sortField === 'format' ? CommonStrings.LABELS.FORMAT : sortField === 'type' ? CommonStrings.LABELS.TYPE : CommonStrings.LABELS.STATUS}
          </Text>
        </Flex>
      </Select.Trigger>
      <Select.Content position="popper" side="bottom" avoidCollisions={false} sideOffset={4}>
        <Select.Item value="name">{CommonStrings.LABELS.NAME}</Select.Item>
        <Select.Item value="format">{CommonStrings.LABELS.FORMAT}</Select.Item>
        <Select.Item value="type">{CommonStrings.LABELS.TYPE}</Select.Item>
        <Select.Item value="status">{CommonStrings.LABELS.STATUS}</Select.Item>
      </Select.Content>
    </Select.Root>
  );

  return (
    <div className="repositories-list">
      <Box
        className="repositories-list__wrapper"
        px={{ initial: '4', md: '6', lg: '6' }}
        py={{ initial: '4', md: '5', lg: '6' }}
        width="100%"
      >
        <Flex direction="column" gap="6" width="100%">
          <Grid
            columns={{ initial: '1', sm: '250px 1fr' }}
            gap="6"
            width="100%"
          >
            {/* Filter Sidebar — hidden on mobile */}
            <Box
              display={{ initial: 'none', sm: 'block' }}
              role="complementary"
              aria-label={RepositoriesPageStrings.LIST.FILTER_BAR_ARIA}
            >
              <aside className="search-sidebar" data-testid="filter-sidebar">
                {filterBarContent}
              </aside>
            </Box>

            {/* Main Content */}
            <Box className="repositories-list__main" minWidth="0" width="100%" role="main">
              <Box className="repositories-list__content">
                <Flex align="baseline" gap="2" mb="4">
                  <Heading as="h1" size="6" weight="bold">{RepositoriesPageStrings.LIST.REPOSITORIES}</Heading>
                  <Text size="2" color="gray">{displayTotal}</Text>
                </Flex>

                {/* Actions Bar */}
                <Box mb="4" role="toolbar">
                  <Flex
                    className="actions-bar"
                    align="center"
                    gap="3"
                    wrap="wrap"
                    style={{ width: '100%' }}
                  >
                    <Box style={{ flex: 1, minWidth: 200 }}>
                      <TextField.Root
                        placeholder={RepositoriesPageStrings.LIST.SEARCH_REPOSITORIES_NAME_PLACEHOLDER}
                        value={filter}
                        onChange={(e) => { setFilter(e.target.value); setCurrentPage(1); }}
                        size="2"
                        style={{ width: '100%' }}
                      >
                        <TextField.Slot>
                          <Search size={14} />
                        </TextField.Slot>
                        {filter && (
                          <TextField.Slot side="right">
                            <IconButton
                              variant="ghost"
                              color="gray"
                              size="1"
                              onClick={() => setFilter('')}
                              aria-label={RepositoriesPageStrings.LIST.CLEAR_FILTER_ARIA}
                            >
                              <X size={14} />
                            </IconButton>
                          </TextField.Slot>
                        )}
                      </TextField.Root>
                    </Box>
                    {renderSortDropdown()}
                  </Flex>
                </Box>

                {/* Results summary */}
                {!loadingRepos && !error && (
                  <Text size="2" color="gray" mb="2" as="p">
                    {RepositoriesPageStrings.LIST.SHOWING_REPOSITORIES(filteredRepositories.length, repositories.length)}
                  </Text>
                )}

                {/* Table */}
                {loadingRepos ? (
                  <Flex direction="column" align="center" justify="center" gap="3" p="9">
                    <Spinner size="3" />
                    <Text color="gray">{RepositoriesPageStrings.LIST.LOADING_REPOSITORIES}</Text>
                  </Flex>
                ) : error ? (
                  <Box p="4">
                    <ErrorState
                      message={error}
                      onRetry={handleRetry}
                    />
                  </Box>
                ) : sortedRepositories.length === 0 ? (
                  emptyState
                ) : (
                  <Table.Root className="repository-list-table" variant="surface" size="2">
                    <Table.Header>
                      <Table.Row>
                        <SortableTableHeader
                          sortKey="name"
                          currentSortKey={sortField}
                          currentSortDirection={sortDirection || 'asc'}
                          onSort={handleSort}
                          align="left"
                        >
                          {CommonStrings.LABELS.NAME}
                        </SortableTableHeader>
                        <Table.ColumnHeaderCell>{RepositoriesPageStrings.LIST.SIZE}</Table.ColumnHeaderCell>
                        <SortableTableHeader
                          sortKey="type"
                          currentSortKey={sortField}
                          currentSortDirection={sortDirection || 'asc'}
                          onSort={handleSort}
                          align="left"
                        >
                          {CommonStrings.LABELS.TYPE}
                        </SortableTableHeader>
                        <SortableTableHeader
                          sortKey="format"
                          currentSortKey={sortField}
                          currentSortDirection={sortDirection || 'asc'}
                          onSort={handleSort}
                          align="left"
                        >
                          {RepositoriesPageStrings.LIST.ECOSYSTEM}
                        </SortableTableHeader>
                        <Table.ColumnHeaderCell>{RepositoriesPageStrings.LIST.BLOB_STORE}</Table.ColumnHeaderCell>
                        <SortableTableHeader
                          sortKey="status"
                          currentSortKey={sortField}
                          currentSortDirection={sortDirection || 'asc'}
                          onSort={handleSort}
                          align="left"
                        >
                          {CommonStrings.LABELS.STATUS}
                        </SortableTableHeader>
                        <Table.ColumnHeaderCell>{RepositoriesPageStrings.LIST.URL}</Table.ColumnHeaderCell>
                        {showHealthCheckColumn && (
                          <Table.ColumnHeaderCell className="table-cell-centered">
                            {RepositoriesPageStrings.LIST.HEALTH_CHECK}
                          </Table.ColumnHeaderCell>
                        )}
                        {showFirewallColumn && (
                          <Table.ColumnHeaderCell className="table-cell-centered">
                            {RepositoriesPageStrings.LIST.FIREWALL_REPORT}
                          </Table.ColumnHeaderCell>
                        )}
                        {/* paddingRight: 32 matches the Rapture-safe right gutter used across list tables (see EntityTable.scss). */}
                        <Table.ColumnHeaderCell
                          width="80px"
                          justify="end"
                          aria-label={RepositoriesPageStrings.LIST.ROW_ACTIONS_ARIA}
                          style={{ paddingRight: 32 }}
                        />
                      </Table.Row>
                    </Table.Header>
                    <Table.Body>
                      {paginatedRepositories.length === 0 ? (
                        <Table.Row>
                          <Table.Cell colSpan={colCount}>
                            <Flex justify="center" p="6">
                              <Text color="gray">
                                {filter || typeFilter.length > 0 || formatFilter.length > 0
                                  ? RepositoriesPageStrings.LIST.NO_REPOSITORIES_MATCH_CURRENT_FILTERS_2
                                  : RepositoriesPageStrings.LIST.NO_REPOSITORIES_AVAILABLE}
                              </Text>
                            </Flex>
                          </Table.Cell>
                        </Table.Row>
                      ) : (
                        paginatedRepositories.map((repo) => (
                          <Table.Row
                            key={repo.name}
                            className="repository-list-table__row repository-list-table__row--clickable"
                            onClick={handleRowClick(repo.name)}
                            onKeyDown={handleRowKeyDown(repo.name)}
                            tabIndex={0}
                            role="button"
                            aria-label={RepositoriesPageStrings.LIST.VIEW(repo.name)}
                          >
                            <Table.Cell>
                              <Text size="2" weight="medium" color="blue">
                                {repo.name}
                              </Text>
                            </Table.Cell>
                            <Table.Cell>
                              <Text size="2" color="gray">
                                {formatFileSize(repo.size)}
                              </Text>
                            </Table.Cell>
                            <Table.Cell>
                              <Text size="2">{repo.type}</Text>
                            </Table.Cell>
                            <Table.Cell>
                              <Text size="2">{FORMAT_LABELS[repo.format] || repo.format}</Text>
                            </Table.Cell>
                            <Table.Cell>
                              <Text size="2" color="gray">
                                {repo.blobStoreName || '—'}
                              </Text>
                            </Table.Cell>
                            <Table.Cell>
                              <Text size="2">{repo.status?.online ? RepositoriesPageStrings.LIST.ONLINE : RepositoriesPageStrings.LIST.OFFLINE}</Text>
                            </Table.Cell>
                            <Table.Cell>
                              <Tooltip content={RepositoriesPageStrings.LIST.COPY_URL_CLIPBOARD}>
                                <IconButton
                                  variant="ghost"
                                  color="gray"
                                  size="1"
                                  onClick={handleCopyUrlButton(repo.url)}
                                  aria-label={RepositoriesPageStrings.LIST.COPY_URL_CLIPBOARD_ARIA}
                                >
                                  <Copy size={16} />
                                </IconButton>
                              </Tooltip>
                            </Table.Cell>
                            {showHealthCheckColumn && (
                              <Table.Cell className="table-cell-centered">
                                <HealthCheckCell
                                  repository={repo}
                                  healthStatus={healthStatus[repo.name]}
                                  onAnalyze={handleAnalyze}
                                  analyzeLoading={analyzingRepos.has(repo.name)}
                                  rhcSupportedByBackend={repo.type === 'proxy' && Object.keys(healthStatus).length > 0 ? repo.name in healthStatus : undefined}
                                />
                              </Table.Cell>
                            )}
                            {showFirewallColumn && (
                              <Table.Cell className="table-cell-centered">
                                <FirewallCell
                                  repository={repo}
                                  firewallStatus={firewallStatus[repo.name]}
                                  firewallLoaded={firewallLoaded}
                                  hasFirewallLicense={true}
                                />
                              </Table.Cell>
                            )}
                            <Table.Cell justify="end" style={{ paddingRight: 32 }}>
                              <DropdownMenu.Root>
                                <DropdownMenu.Trigger>
                                  <IconButton
                                    variant="ghost"
                                    color="gray"
                                    size="1"
                                    onClick={(e) => e.stopPropagation()}
                                    aria-label={RepositoriesPageStrings.LIST.ACTIONS_FOR(repo.name)}
                                  >
                                    <MoreHorizontal size={16} />
                                  </IconButton>
                                </DropdownMenu.Trigger>
                                <DropdownMenu.Content align="end" onClick={(e) => e.stopPropagation()}>
                                  {canEdit && (
                                    <DropdownMenu.Item
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        onSelect(repo.name);
                                      }}
                                      data-testid={`repo-action-edit-${repo.name}`}
                                    >
                                      {CommonStrings.ACTIONS.EDIT}
                                    </DropdownMenu.Item>
                                  )}
                                  {canEdit && (repo.type === 'hosted' || repo.type === 'proxy') && (
                                    <DropdownMenu.Item
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        stageRebuildIndex(repo);
                                      }}
                                      data-testid={`repo-action-rebuild-index-${repo.name}`}
                                    >
                                      {RepositoriesPageStrings.LIST.REBUILD_INDEX}
                                    </DropdownMenu.Item>
                                  )}
                                  {canEdit && (repo.type === 'proxy' || repo.type === 'group') && (
                                    <DropdownMenu.Item
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        stageInvalidateCache(repo);
                                      }}
                                      data-testid={`repo-action-invalidate-cache-${repo.name}`}
                                    >
                                      {RepositoriesPageStrings.LIST.INVALIDATE_CACHE}
                                    </DropdownMenu.Item>
                                  )}
                                  {canEdit && (
                                    <DropdownMenu.Item
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        stageToggleOnline(repo);
                                      }}
                                      data-testid={`repo-action-toggle-online-${repo.name}`}
                                    >
                                      {(repo.status?.online ?? repo.online ?? true)
                                        ? RepositoriesPageStrings.LIST.DISABLE_SYSTEM_STATUS
                                        : RepositoriesPageStrings.LIST.ENABLE_SYSTEM_STATUS}
                                    </DropdownMenu.Item>
                                  )}
                                  <DropdownMenu.Item
                                    onClick={handleCopyUrlButton(repo.url)}
                                    data-testid={`repo-action-copy-url-${repo.name}`}
                                  >
                                    {RepositoriesPageStrings.LIST.COPY_URL}
                                  </DropdownMenu.Item>
                                  {canDelete && (
                                    <DropdownMenu.Item
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        setRepoToDelete(repo);
                                        setDeleteModalOpen(true);
                                      }}
                                      color="red"
                                      data-testid={`repo-action-delete-${repo.name}`}
                                    >
                                      {CommonStrings.ACTIONS.DELETE}
                                    </DropdownMenu.Item>
                                  )}
                                </DropdownMenu.Content>
                              </DropdownMenu.Root>
                            </Table.Cell>
                          </Table.Row>
                        ))
                      )}
                    </Table.Body>
                  </Table.Root>
                )}

                {/* Pagination */}
                {!loadingRepos && !error && sortedRepositories.length > 0 && (
                  <Box className="repositories-list__pagination" p="3">
                    <TablePagination
                      currentPage={currentPage}
                      totalPages={totalPages}
                      itemsPerPage={pageSize}
                      totalItems={sortedRepositories.length}
                      onPageChange={handlePageChange}
                      onItemsPerPageChange={handlePageSizeChange}
                      mt="0"
                    />
                  </Box>
                )}

                {/* Help Section */}
                <HelpSection
                  title={RepositoriesPageStrings.LIST.ABOUT_REPOSITORIES}
                  content={RepositoriesPageStrings.LIST.REPOSITORIES_STORAGE_LOCATIONS_WHERE_SOFTWARE}
                  docLink={{
                    label: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
                    href: 'https://help.globalart.dev/en/repository-administration.html',
                  }}
                  className="repositories-list__help"
                />
              </Box>
            </Box>
          </Grid>
        </Flex>
      </Box>

      {/* Delete Confirmation Modal */}
      <DeleteConfirmationModal
        open={deleteModalOpen}
        onClose={() => setDeleteModalOpen(false)}
        onConfirm={handleDeleteConfirm}
        entityName={repoToDelete?.name || ''}
        entityType="repository"
        loading={isDeleting}
        storageSize={repoToDelete?.size != null ? formatFileSize(repoToDelete.size) : undefined}
      />
      <ConfirmDialog
        open={pendingRowAction !== null}
        onOpenChange={(open) => { if (!open) cancelPendingRowAction(); }}
        title={rowActionDialogProps.title}
        message={rowActionDialogProps.message}
        confirmLabel={rowActionDialogProps.confirmLabel}
        variant={rowActionDialogProps.variant}
        onConfirm={handleConfirmRowAction}
      />
    </div>
  );
}

export default RepositoriesList;
