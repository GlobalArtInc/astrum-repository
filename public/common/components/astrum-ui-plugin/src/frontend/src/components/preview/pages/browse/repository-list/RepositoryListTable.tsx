

import React, { useCallback } from 'react';
import {
  Badge,
  Box,
  Callout,
  DropdownMenu,
  Flex,
  IconButton,
  Spinner,
  Table,
  Text,
} from '@radix-ui/themes';
import { MoreHorizontal, X } from 'lucide-react';
import { useRouter } from '@uirouter/react';

import type { Repository, HealthCheckStatus, FirewallStatus } from './repository-list.types';
import { HealthCheckCell } from '../../../shared/security/HealthCheckCell';
import { FirewallCell } from '../../../shared/security/FirewallCell';
import {
  SortableTableHeader,
  useToast,
  type SortDirection,
} from '../../../shared';
import { ensureTrailingSlash } from '../../../../../utils/url';
import UIStrings from '../../../constants/UIStrings';
import { CopyButton, copyToClipboard } from '../../../shared/Usage';
import { FormatBadge, StatusLabel } from '../shared';
import { canOpenRepositorySettings } from '../RepositoryHeader';

import './RepositoryListTable.scss';

/**
 * The path of a repository URL (`/repository/npm-hosted/`); the host is the same for every row.
 */
function urlPath(url: string): string {
  try {
    return new URL(url).pathname;
  } catch {
    return url;
  }
}

const TYPE_KEYS: Record<string, 'HOSTED' | 'PROXY' | 'GROUP'> = {
  hosted: 'HOSTED',
  proxy: 'PROXY',
  group: 'GROUP',
};

export interface RepositoryListTableProps {
  /** Repositories to display */
  repositories: readonly Repository[];
  /** Loading state */
  loading?: boolean;
  /** Error message */
  error?: string;
  /** Callback when a repository is selected */
  onSelect?: (repositoryName: string) => void;
  /** Health check statuses by repository name */
  healthCheck?: Record<string, HealthCheckStatus>;
  /** Firewall statuses by repository name */
  firewallStatus?: Record<string, FirewallStatus>;
  /** Whether the firewall summary API has finished loading */
  firewallLoaded?: boolean;
  /** Show health check column */
  showHealthCheck?: boolean;
  /** Show IQ policy violations column */
  showIqPolicyViolations?: boolean;
  /** Whether any filters are currently active */
  hasFilters?: boolean;
  /** Callback when analyze is clicked */
  onAnalyze?: (repositoryName: string) => void;
  /** Set of repository names being analyzed */
  analyzingRepos?: Set<string>;
  /** Whether user has Firewall license */
  hasFirewallLicense?: boolean;
  /** Summary of proxy protection status */
  proxyProtectionSummary?: { totalProxy: number; protectedProxy: number };
  /** Current sort field (for server-side sort) */
  sortKey?: string;
  /** Current sort direction */
  sortDirection?: SortDirection;
  /** Callback when sort changes (from header click) */
  onSort?: (key: string, direction: SortDirection) => void;
}

/**
 * RepositoryListTable - the repositories with their format, type, status and URL.
 */
export function RepositoryListTable({
  repositories,
  loading = false,
  error,
  onSelect,
  healthCheck = {},
  firewallStatus = {},
  firewallLoaded = false,
  showHealthCheck = false,
  showIqPolicyViolations = false,
  hasFilters = false,
  onAnalyze,
  analyzingRepos = new Set(),
  hasFirewallLicense = true,
  proxyProtectionSummary,
  sortKey = 'name',
  sortDirection = 'asc',
  onSort,
}: RepositoryListTableProps): JSX.Element {
  const toast = useToast();
  const router = useRouter();
  const strings = UIStrings.BROWSE.LIST;

  // Handle sort - must be declared before any early returns (Rules of Hooks)
  const handleSort = useCallback(
    (key: string, direction: SortDirection) => {
      onSort?.(key, direction);
    },
    [onSort],
  );

  // Handle row click
  const handleRowClick = (name: string) => () => {
    onSelect?.(name);
  };

  // Handle row keyboard navigation
  const handleRowKeyDown = (name: string) => (e: React.KeyboardEvent) => {
    if (e.target !== e.currentTarget) {
      return;
    }
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onSelect?.(name);
    }
  };

  // Handle copy URL from the row menu
  const handleCopyUrl = (url: string) => async (e: React.MouseEvent | Event) => {
    e.stopPropagation();
    if (await copyToClipboard(ensureTrailingSlash(url))) {
      toast.success(strings.URL_COPIED);
    } else {
      toast.error(strings.URL_COPY_FAILED);
    }
  };

  // Loading state
  if (loading) {
    return (
      <Flex direction="column" align="center" justify="center" gap="3" p="9">
        <Spinner size="3" />
        <Text color="gray">{strings.LOADING}</Text>
      </Flex>
    );
  }

  // Error state
  if (error) {
    return (
      <Box p="4">
        <Callout.Root color="red">
          <Callout.Icon>
            <X size={16} />
          </Callout.Icon>
          <Callout.Text>{error}</Callout.Text>
        </Callout.Root>
      </Box>
    );
  }

  // Column count for empty state: name, format, type, status, url, + optional (health, IQ), + actions.
  const colCount = 6 + (showHealthCheck ? 1 : 0) + (showIqPolicyViolations ? 1 : 0);

  const header = (key: string, label: string) =>
    onSort ? (
      <SortableTableHeader
        sortKey={key}
        currentSortKey={sortKey}
        currentSortDirection={sortDirection}
        onSort={handleSort}
        align="left"
      >
        {label}
      </SortableTableHeader>
    ) : (
      <Table.ColumnHeaderCell>{label}</Table.ColumnHeaderCell>
    );

  return (
    <Table.Root className="repository-list-table" variant="surface" size="2">
      <Table.Header>
        <Table.Row>
          {header('name', strings.COLUMNS.NAME)}
          {header('format', strings.COLUMNS.FORMAT)}
          {header('type', strings.COLUMNS.TYPE)}
          {header('status', strings.COLUMNS.STATUS)}
          <Table.ColumnHeaderCell className="repository-list-table__url-header">
            {strings.COLUMNS.URL}
          </Table.ColumnHeaderCell>
          {showHealthCheck && (
            <Table.ColumnHeaderCell className="table-cell-centered">
              {strings.COLUMNS.HEALTH_CHECK}
            </Table.ColumnHeaderCell>
          )}
          {showIqPolicyViolations && (
            <Table.ColumnHeaderCell className="table-cell-centered">
              {strings.COLUMNS.FIREWALL}
            </Table.ColumnHeaderCell>
          )}
          <Table.ColumnHeaderCell
            width="56px"
            justify="end"
            aria-label={strings.COLUMNS.ACTIONS}
          />
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {repositories.length === 0 ? (
          <Table.Row>
            <Table.Cell colSpan={colCount}>
              <Flex justify="center" p="6">
                <Text color="gray">
                  {hasFilters ? strings.EMPTY_FILTERED : strings.EMPTY}
                </Text>
              </Flex>
            </Table.Cell>
          </Table.Row>
        ) : (
          repositories.map((repo) => {
            const typeKey = TYPE_KEYS[repo.type];
            const url = ensureTrailingSlash(repo.url ?? '');
            return (
              <Table.Row
                key={repo.name}
                className="repository-list-table__row repository-list-table__row--clickable"
                onClick={handleRowClick(repo.name)}
                onKeyDown={handleRowKeyDown(repo.name)}
                tabIndex={0}
                role="button"
                aria-label={strings.ROW_LABEL(repo.name)}
                data-analytics-id="as-browse-select-repository"
              >
                <Table.Cell>
                  <Text size="2" weight="medium" className="repository-list-table__name">
                    {repo.name}
                  </Text>
                </Table.Cell>
                <Table.Cell>
                  <FormatBadge format={repo.format} size="1" radius="full" />
                </Table.Cell>
                <Table.Cell>
                  <Badge color="gray" variant="soft" size="1" radius="full" className="repository-list-table__type">
                    {typeKey ? UIStrings.BROWSE.TYPES[typeKey] : repo.type}
                  </Badge>
                </Table.Cell>
                <Table.Cell>
                  <StatusLabel
                    online={Boolean(repo.status?.online)}
                    description={repo.status?.description || repo.status?.reason}
                  />
                </Table.Cell>
                <Table.Cell className="repository-list-table__url-cell">
                  {repo.url && (
                    <Flex align="center" gap="1" minWidth="0">
                      <Text size="1" color="gray" className="repository-list-table__url" title={url}>
                        {urlPath(url)}
                      </Text>
                      <CopyButton
                        text={url}
                        iconOnly
                        size="1"
                        label={strings.COPY_URL}
                        onCopied={() => toast.success(strings.URL_COPIED)}
                        onCopyFailed={() => toast.error(strings.URL_COPY_FAILED)}
                      />
                    </Flex>
                  )}
                </Table.Cell>
                {showHealthCheck && (
                  <Table.Cell className="table-cell-centered">
                    <HealthCheckCell
                      repository={repo}
                      healthStatus={healthCheck[repo.name]}
                      onAnalyze={onAnalyze}
                      analyzeLoading={analyzingRepos.has(repo.name)}
                      rhcSupportedByBackend={repo.type === 'proxy' && Object.keys(healthCheck).length > 0 ? repo.name in healthCheck : undefined}
                    />
                  </Table.Cell>
                )}
                {showIqPolicyViolations && (
                  <Table.Cell className="table-cell-centered">
                    <FirewallCell
                      repository={repo}
                      firewallStatus={firewallStatus[repo.name]}
                      firewallLoaded={firewallLoaded}
                      hasFirewallLicense={hasFirewallLicense}
                      proxyProtectionSummary={proxyProtectionSummary}
                    />
                  </Table.Cell>
                )}
                <Table.Cell justify="end">
                  <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                      <IconButton
                        variant="ghost"
                        color="gray"
                        size="1"
                        onClick={(e) => e.stopPropagation()}
                        aria-label={strings.ROW_ACTIONS(repo.name)}
                      >
                        <MoreHorizontal size={16} />
                      </IconButton>
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content align="end" onClick={(e) => e.stopPropagation()}>
                      <DropdownMenu.Item
                        onClick={(e) => {
                          e.stopPropagation();
                          onSelect?.(repo.name);
                        }}
                        data-analytics-id="as-browse-repository-menu"
                      >
                        {strings.BROWSE_ACTION}
                      </DropdownMenu.Item>
                      <DropdownMenu.Item
                        onClick={handleCopyUrl(repo.url)}
                        data-analytics-id="as-repository-copy-url-menu"
                      >
                        {strings.COPY_URL}
                      </DropdownMenu.Item>
                      {canOpenRepositorySettings(repo) && (
                        <DropdownMenu.Item
                          onClick={(e) => {
                            e.stopPropagation();
                            router.stateService.go('preview.admin.repository.repositories.detail', {
                              repositoryName: repo.name,
                            });
                          }}
                        >
                          {strings.SETTINGS_ACTION}
                        </DropdownMenu.Item>
                      )}
                    </DropdownMenu.Content>
                  </DropdownMenu.Root>
                </Table.Cell>
              </Table.Row>
            );
          })
        )}
      </Table.Body>
    </Table.Root>
  );
}

export default RepositoryListTable;
