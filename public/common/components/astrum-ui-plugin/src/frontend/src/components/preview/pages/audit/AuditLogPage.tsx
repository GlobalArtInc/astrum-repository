

import React, { useState, useCallback, useDeferredValue, useEffect } from 'react';
import {
  Box,
  Button,
  Callout,
  Card,
  Flex,
  Grid,
  Heading,
  IconButton,
  Spinner,
  Table,
  Text,
  Code,
  Badge,
  Inset,
  TextField,
} from '@radix-ui/themes';
import { AlertCircle, RefreshCw, ChevronDown, ChevronUp, Search, X, Filter, Info } from 'lucide-react';
import { useCurrentStateAndParams } from '@uirouter/react';

import type { AuditFilters, AuditCategory } from '@globalart/astrum-ui-plugin';
import {
  useAuditLogApi,
  formatAuditEvent,
  formatTimestamp,
  CATEGORY_COLORS,
  CATEGORY_LABELS,
} from '@globalart/astrum-ui-plugin';
import { AuditFilterSidebar } from './AuditFilterSidebar';
import { AUDIT_LOG_STRINGS } from './strings';
import { MobileFilterDrawer, TablePagination, useRepositoriesApi } from '@globalart/astrum-ui-plugin';
import { PageHeader } from '../../shared/PageHeader/PageHeader';
import type { Repository } from '../settings/repository/repositories/types';
import '@globalart/astrum-ui-plugin/sidebar.scss';
import AuditStrings from '../../constants/pages/audit/AuditStrings';
import CommonStrings from '../../constants/CommonStrings';
import plural, { pageLanguage } from '../../../../interface/plural';

const DEFAULT_FILTERS: AuditFilters = {
  categories: [],
  domains: [],
  eventTypes: [],
  dateRange: 'last-30-days',
  initiator: '',
  initiators: [],
  searchQuery: '',
  repositoryName: '',
  repositoryType: '',
};

const LARGE_RESULT_SET_PAGE_THRESHOLD = 1000;

/**
 * Audit Log Page - Displays comprehensive audit events across all system activities.
 *
 * Shows security, repository, configuration, and protection events with filtering,
 * pagination, and expandable details.
 */
export function AuditLogPage() {
  const { params } = useCurrentStateAndParams();
  const [filters, setFilters] = useState<AuditFilters>({
    ...DEFAULT_FILTERS,
    repositoryName: (params?.repositoryName as string) || '',
  });
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage, setItemsPerPage] = useState(20);
  const [expandedRows, setExpandedRows] = useState<Set<number>>(new Set());
  const [showMobileFilters, setShowMobileFilters] = useState(false);
  const [searchInput, setSearchInput] = useState('');
  const debouncedSearch = useDeferredValue(searchInput);

  const { data, loading, error, refetch } = useAuditLogApi({
    filters,
    page: currentPage,
    limit: itemsPerPage,
  });

  const { fetchRepositories } = useRepositoriesApi();
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [repositoryError, setRepositoryError] = useState<string | null>(null);

  useEffect(() => {
    fetchRepositories()
      .then(setRepositories)
      .catch((err: Error) => setRepositoryError(err.message));
  }, [fetchRepositories]);

  // Apply debounced search to filters
  useEffect(() => {
    setFilters((f) => ({ ...f, searchQuery: debouncedSearch }));
    setCurrentPage(1);
  }, [debouncedSearch]);

  const toggleRow = (id: number) => {
    const newExpanded = new Set(expandedRows);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedRows(newExpanded);
  };

  const handleCategoryToggle = useCallback(
    (category: AuditCategory) => {
      const newCategories = filters.categories.includes(category)
        ? filters.categories.filter((c) => c !== category)
        : [...filters.categories, category];
      setFilters({ ...filters, categories: newCategories });
      setCurrentPage(1);
    },
    [filters]
  );

  const handleEventTypeToggle = useCallback(
    (eventType: string) => {
      const newEventTypes = filters.eventTypes.includes(eventType)
        ? filters.eventTypes.filter((t) => t !== eventType)
        : [...filters.eventTypes, eventType];
      setFilters({ ...filters, eventTypes: newEventTypes });
      setCurrentPage(1);
    },
    [filters]
  );

  const handleInitiatorChange = useCallback(
    (initiator: string) => {
      setFilters({ ...filters, initiator });
      setCurrentPage(1);
    },
    [filters]
  );

  const handleRepositoryNameChange = useCallback(
    (repositoryName: string) => {
      setFilters({ ...filters, repositoryName });
      setCurrentPage(1);
    },
    [filters]
  );

  const handleRepositoryTypeChange = useCallback(
    (repositoryType: string) => {
      setFilters({ ...filters, repositoryType });
      setCurrentPage(1);
    },
    [filters]
  );

  const handleDateRangeChange = useCallback(
    (dateRange: AuditFilters['dateRange']) => {
      setFilters({ ...filters, dateRange });
      setCurrentPage(1);
    },
    [filters]
  );

  const handleSearchChange = useCallback((searchQuery: string) => {
    setSearchInput(searchQuery);
  }, []);

  const handleClearAllFilters = useCallback(() => {
    setFilters(DEFAULT_FILTERS);
    setSearchInput('');
    setCurrentPage(1);
  }, []);

  const hasActiveFilters =
    filters.categories.length > 0 ||
    filters.eventTypes.length > 0 ||
    filters.dateRange !== 'last-30-days' ||
    filters.initiator !== '' ||
    filters.initiators.length > 0;

  const filterBarContent = (
    <AuditFilterSidebar
      filters={filters}
      repositories={repositories}
      onCategoryToggle={handleCategoryToggle}
      onEventTypeToggle={handleEventTypeToggle}
      onInitiatorChange={handleInitiatorChange}
      onRepositoryNameChange={handleRepositoryNameChange}
      onRepositoryTypeChange={handleRepositoryTypeChange}
      onDateRangeChange={handleDateRangeChange}
      onClearAllFilters={handleClearAllFilters}
      disabled={loading}
    />
  );

  return (
    <Box
      data-testid="audit-log-page"
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
      width="100%"
      style={{ minWidth: 0, boxSizing: 'border-box' }}
    >
      <Grid columns={{ initial: '1', sm: '280px 1fr' }} gap="4" width="100%" style={{ minWidth: 0 }}>
        {/* Filter Sidebar - hidden on mobile */}
        <Box
          display={{ initial: 'none', sm: 'block' }}
          style={{ overflow: 'visible', minWidth: 0 }}
          data-testid="filter-sidebar"
          role="complementary"
          aria-label={AuditStrings.PAGE.AUDIT_LOG_FILTERS_ARIA}
        >
          {filterBarContent}
        </Box>

        {/* Main Content */}
        <Box className="page-content" minWidth="0" width="100%" role="main" aria-label={AuditStrings.PAGE.PAGE_CONTENT_ARIA}>
          <Box mb="4">
            <PageHeader
              title={AuditStrings.PAGE.AUDIT_LOG}
              description={
                data
                  ? plural(AuditStrings.PAGE.EVENT_COUNT, data.pagination.totalItems, data.pagination.totalItems.toLocaleString(pageLanguage()))
                  : undefined
              }
              actions={
                <Flex align="center" gap="3">
                  <Button variant="ghost" size="2" onClick={refetch} disabled={loading}>
                    <RefreshCw size={16} />
                    {CommonStrings.ACTIONS.REFRESH}
                  </Button>
                </Flex>
              }
            />
          </Box>

          {/* Search and Mobile Filter Bar */}
          <Box mb="4" role="toolbar" aria-label={AuditStrings.PAGE.ACTIONS_BAR_ARIA}>
            <Flex
              className="actions-bar"
              align="center"
              gap="3"
              wrap="wrap"
              style={{ width: '100%' }}
            >
              <Box style={{ flex: 1, minWidth: 200 }}>
                <TextField.Root
                  placeholder={AuditStrings.PAGE.SEARCH_CONTEXT_E_G_USER_PLACEHOLDER}
                  value={searchInput}
                  onChange={(e) => handleSearchChange(e.target.value)}
                  size="2"
                  style={{ width: '100%' }}
                >
                  <TextField.Slot>
                    <Search size={14} />
                  </TextField.Slot>
                  {searchInput && (
                    <TextField.Slot side="right">
                      <IconButton
                        variant="ghost"
                        color="gray"
                        size="1"
                        onClick={() => handleSearchChange('')}
                        aria-label={AuditStrings.PAGE.CLEAR_SEARCH_ARIA}
                      >
                        <X size={14} />
                      </IconButton>
                    </TextField.Slot>
                  )}
                </TextField.Root>
              </Box>
              <Box display={{ initial: 'flex', sm: 'none' }}>
                <Button
                  variant="outline"
                  size="2"
                  color="gray"
                  onClick={() => setShowMobileFilters(true)}
                  aria-label={AuditStrings.PAGE.OPEN_FILTERS_ARIA}
                >
                  <Filter size={16} />
                  {CommonStrings.ACTIONS.FILTER}
                </Button>
              </Box>
            </Flex>
          </Box>

          {repositoryError && (
            <Callout.Root color="red" mb="4">
              <Callout.Icon><AlertCircle size={16} /></Callout.Icon>
              <Callout.Text>{AuditStrings.PAGE.FAILED_LOAD_REPOSITORIES(repositoryError)}</Callout.Text>
            </Callout.Root>
          )}

          {error && (
            <Callout.Root color="red" mb="4">
              <Callout.Icon><AlertCircle size={16} /></Callout.Icon>
              <Callout.Text>{error}</Callout.Text>
            </Callout.Root>
          )}

          {data &&
            data.pagination.totalItems >
              LARGE_RESULT_SET_PAGE_THRESHOLD * data.pagination.itemsPerPage && (
              <Callout.Root role="status" color="blue" mb="4">
                <Callout.Icon><Info size={16} /></Callout.Icon>
                <Callout.Text>
                  {AUDIT_LOG_STRINGS.LARGE_RESULT_SET_HINT}
                </Callout.Text>
              </Callout.Root>
            )}

          {loading ? (
            <Flex justify="center" align="center" style={{ minHeight: '400px' }}>
              <Flex direction="column" align="center" gap="3">
                <Spinner size="3" />
                <Text color="gray">{AuditStrings.PAGE.LOADING_AUDIT_EVENTS}</Text>
              </Flex>
            </Flex>
          ) : data && data.items.length > 0 ? (
            <>
              <Card>
                <Inset>
                  <Box style={{ overflowX: 'auto' }}>
                    <Table.Root size="2">
                      <Table.Header>
                        <Table.Row>
                          <Table.ColumnHeaderCell style={{ width: '40px' }}></Table.ColumnHeaderCell>
                          <Table.ColumnHeaderCell>{AuditStrings.PAGE.TIMESTAMP}</Table.ColumnHeaderCell>
                          <Table.ColumnHeaderCell>{AuditStrings.PAGE.CATEGORY}</Table.ColumnHeaderCell>
                          <Table.ColumnHeaderCell>{AuditStrings.PAGE.EVENT}</Table.ColumnHeaderCell>
                          <Table.ColumnHeaderCell>{AuditStrings.PAGE.SUMMARY}</Table.ColumnHeaderCell>
                          <Table.ColumnHeaderCell>{AuditStrings.PAGE.INITIATOR}</Table.ColumnHeaderCell>
                        </Table.Row>
                      </Table.Header>

                      <Table.Body>
                        {data.items.map((event) => {
                          const displayEvent = formatAuditEvent(event);
                          const isExpanded = expandedRows.has(event.id);

                          return (
                            <React.Fragment key={event.id}>
                              <Table.Row
                                style={{ cursor: 'pointer' }}
                                onClick={() => toggleRow(event.id)}
                              >
                                <Table.Cell>
                                  {isExpanded ? (
                                    <ChevronUp size={16} />
                                  ) : (
                                    <ChevronDown size={16} />
                                  )}
                                </Table.Cell>
                                <Table.Cell style={{ whiteSpace: 'nowrap' }}>
                                  <Text size="2">{formatTimestamp(event.timestamp)}</Text>
                                </Table.Cell>
                                <Table.Cell>
                                  <Badge
                                    color={CATEGORY_COLORS[displayEvent.category] as any}
                                    variant="soft"
                                    size="1"
                                  >
                                    {CATEGORY_LABELS[displayEvent.category]}
                                  </Badge>
                                </Table.Cell>
                                <Table.Cell>
                                  <Text size="2" weight="medium">
                                    {displayEvent.eventLabel}
                                  </Text>
                                </Table.Cell>
                                <Table.Cell>
                                  <Text size="2">{displayEvent.summary}</Text>
                                </Table.Cell>
                                <Table.Cell>
                                  <Text size="2" color="gray">
                                    {event.initiator || 'system'}
                                  </Text>
                                </Table.Cell>
                              </Table.Row>

                              {isExpanded && (
                                <Table.Row>
                                  <Table.Cell colSpan={6}>
                                    <Box p="4" style={{ backgroundColor: 'var(--gray-2)' }}>
                                      <Heading size="3" mb="2">
                                        {AuditStrings.PAGE.EVENT_DETAILS}
                                      </Heading>
                                      <Flex direction="column" gap="2" mb="3">
                                        <Text size="2">
                                          <strong>{AuditStrings.PAGE.DOMAIN}</strong> {event.domain}
                                        </Text>
                                        <Text size="2">
                                          <strong>{AuditStrings.PAGE.TYPE}</strong> {event.type}
                                        </Text>
                                        <Text size="2">
                                          <strong>{AuditStrings.PAGE.CONTEXT}</strong> {event.context || '-'}
                                        </Text>
                                        <Text size="2">
                                          <strong>{AuditStrings.PAGE.NODE_ID}</strong> {event.nodeId}
                                        </Text>
                                      </Flex>

                                      <Text size="2" weight="medium" mb="1">
                                        {AuditStrings.PAGE.ATTRIBUTES}
                                      </Text>
                                      <Code
                                        size="2"
                                        style={{
                                          display: 'block',
                                          whiteSpace: 'pre',
                                          overflowX: 'auto',
                                          padding: '12px',
                                        }}
                                      >
                                        {JSON.stringify(event.attributes, null, 2)}
                                      </Code>
                                    </Box>
                                  </Table.Cell>
                                </Table.Row>
                              )}
                            </React.Fragment>
                          );
                        })}
                      </Table.Body>
                    </Table.Root>
                  </Box>
                </Inset>
              </Card>

              <TablePagination
                currentPage={currentPage}
                totalPages={data.pagination.totalPages}
                itemsPerPage={itemsPerPage}
                totalItems={data.pagination.totalItems}
                onPageChange={setCurrentPage}
                onItemsPerPageChange={setItemsPerPage}
              />
            </>
          ) : (
            <Card>
              <Flex direction="column" align="center" justify="center" gap="2" py="8">
                <Text size="3" weight="medium">
                  {AuditStrings.PAGE.NO_AUDIT_EVENTS_FOUND}
                </Text>
                <Text size="2" color="gray">
                  {hasActiveFilters || filters.searchQuery
                    ? AuditStrings.PAGE.NO_AUDIT_EVENTS_MATCH_CURRENT
                    : AuditStrings.PAGE.NO_AUDIT_EVENTS_HAVE_BEEN}
                </Text>
                {(hasActiveFilters || filters.searchQuery) && (
                  <Button variant="outline" size="2" mt="2" onClick={handleClearAllFilters}>
                    {AuditStrings.PAGE.CLEAR_ALL_FILTERS}
                  </Button>
                )}
              </Flex>
            </Card>
          )}
        </Box>
      </Grid>

      <MobileFilterDrawer
        isOpen={showMobileFilters}
        onClose={() => setShowMobileFilters(false)}
        title={CommonStrings.ACTIONS.FILTER}
        onClearAll={handleClearAllFilters}
      >
        <AuditFilterSidebar
          filters={filters}
          repositories={repositories}
          onCategoryToggle={handleCategoryToggle}
          onEventTypeToggle={handleEventTypeToggle}
          onInitiatorChange={handleInitiatorChange}
          onRepositoryNameChange={handleRepositoryNameChange}
          onRepositoryTypeChange={handleRepositoryTypeChange}
          onDateRangeChange={handleDateRangeChange}
          onClearAllFilters={handleClearAllFilters}
          disabled={loading}
        />
      </MobileFilterDrawer>
    </Box>
  );
}

export default AuditLogPage;
