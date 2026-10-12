

import React, { useState, useMemo, useCallback, useRef } from 'react';
import {
  Badge,
  Box,
  Button,
  Card,
  Flex,
  Heading,
  ScrollArea,
  Select,
  Spinner,
  Table,
  Text,
  TextField,
  Tooltip,
} from '@radix-ui/themes';
import {
  Calendar,
  ChevronDown,
  ChevronUp,
  Clock,
  Download,
  ExternalLink,
  Package,
  Search,
  Tag,
  Trash2,
  X,
} from 'lucide-react';
import { useRouter, useCurrentStateAndParams } from '@uirouter/react';

import { useToast } from '../../shared/Toast';
import { ConfirmDialog } from '../../shared/form';
import { exportToCsv, PageHeader, useRouteVisibility } from '../../shared';
import { ExtJS } from '../../../../interface/ExtJS';
import Permissions from '../../../../constants/Permissions';

import { useTagDetailExtended, type TaggedComponent } from './hooks';
import { useSearchNavigation } from '../search/unified/useSearchNavigation';

import './TagDetailPage.scss';
import TagsStrings from '../../constants/pages/tags/TagsStrings';
import CommonStrings from '../../constants/CommonStrings';
import { pageLanguage } from '../../../../interface/plural';

/**
 * Sort configuration.
 */
type SortField = 'name' | 'format' | 'version' | 'repository';
type SortDirection = 'asc' | 'desc';

/**
 * UI Strings.
 */
const STRINGS = {
  backToTags: TagsStrings.DETAIL.TAGS,
  loading: TagsStrings.DETAIL.LOADING_TAG_DETAILS,
  loadingComponents: TagsStrings.DETAIL.LOADING_TAGGED_COMPONENTS,
  error: TagsStrings.DETAIL.FAILED_LOAD_TAG_DETAILS,
  retry: CommonStrings.ACTIONS.RETRY,

  header: {
    created: TagsStrings.DETAIL.CREATED,
    lastUpdated: TagsStrings.DETAIL.LAST_UPDATED,
    totalComponents: TagsStrings.DETAIL.TOTAL_COMPONENTS,
  },

  components: {
    title: TagsStrings.DETAIL.TAGGED_COMPONENTS,
    searchPlaceholder: TagsStrings.DETAIL.SEARCH_COMPONENTS,
    formatFilter: CommonStrings.LABELS.FORMAT,
    repositoryFilter: TagsStrings.DETAIL.REPOSITORY,
    allFormats: TagsStrings.DETAIL.ALL_FORMATS,
    allRepositories: TagsStrings.DETAIL.ALL_REPOSITORIES,
    exportCsv: TagsStrings.DETAIL.EXPORT_CSV,
    empty: TagsStrings.DETAIL.NO_COMPONENTS_TAGGED_TAG_YET,
    emptyFiltered: TagsStrings.DETAIL.NO_COMPONENTS_MATCH_FILTERS,
    error: TagsStrings.DETAIL.FAILED_LOAD_TAGGED_COMPONENTS,
    loadMoreError: TagsStrings.DETAIL.FAILED_LOAD_MORE_COMPONENTS,
    retry: CommonStrings.ACTIONS.RETRY,
  },

  table: {
    component: TagsStrings.DETAIL.COMPONENT,
    format: 'Format',
    version: TagsStrings.DETAIL.VERSION,
    repository: TagsStrings.DETAIL.REPOSITORY,
    actions: '',
  },

  actions: {
    view: TagsStrings.DETAIL.VIEW_DETAILS,
    remove: TagsStrings.DETAIL.REMOVE_TAG,
    delete: TagsStrings.DETAIL.DELETE_TAG,
  },

  deleteDialog: {
    title: TagsStrings.DETAIL.DELETE_TAG,
    message: TagsStrings.DETAIL.YOU_SURE_YOU_WANT_DELETE,
    warning: TagsStrings.DETAIL.REMOVE_TAG_ALL_ASSOCIATED_COMPONENTS,
    cancel: CommonStrings.ACTIONS.CANCEL,
    confirm: CommonStrings.ACTIONS.DELETE,
  },
};

/**
 * Format date for display.
 */
function formatDate(dateString: string | null): string {
  if (!dateString) return '-';
  try {
    return new Date(dateString).toLocaleDateString(pageLanguage(), {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  } catch {
    return dateString;
  }
}

/**
 * Get format badge color.
 */
function getFormatColor(format: string): 'red' | 'blue' | 'purple' | 'cyan' | 'orange' | 'green' | 'gray' {
  const colorMap: Record<string, 'red' | 'blue' | 'purple' | 'cyan' | 'orange' | 'green' | 'gray'> = {
    maven2: 'red',
    npm: 'red',
    nuget: 'blue',
    pypi: 'blue',
    docker: 'cyan',
    helm: 'purple',
    go: 'cyan',
    rubygems: 'red',
    yum: 'orange',
    apt: 'purple',
    raw: 'gray',
  };
  return colorMap[format.toLowerCase()] || 'gray';
}

/**
 * TagDetailPage - Full page view of a single tag with its components.
 *
 * This component follows the three-layer architecture:
 * - Layer 1: tagDetailMachine (XState state machine)
 * - Layer 2: useTagDetailExtended (integration hook)
 * - Layer 3: TagDetailPage (presentation component)
 *
 * The component contains only presentation logic:
 * - Rendering UI based on state
 * - Handling user input events
 * - Displaying validation errors
 * - Showing loading/error states
 */
export function TagDetailPage(): JSX.Element {
  const router = useRouter();
  const { params } = useCurrentStateAndParams();
  const tagName = params.tagName as string;
  const toast = useToast();
  const { navigateToDetail } = useSearchNavigation();

  // Check if user can navigate to Search component detail (same check the left-nav uses)
  const canOpenComponent = useRouteVisibility('preview.browse.search');

  // Gate the Delete Tag action on the tags:delete permission (ASTRUM-54212). This prominent
  // destructive button stays visible but disabled for read-only users (discoverability),
  // rather than being hidden like the inline edit/save controls — a deliberate asymmetry
  // matching the Classic UI's large action buttons.
  // Use the provider-independent ExtJS.usePermission: the context-based usePermission
  // returns false without a <PermissionsProvider> ancestor, which coreui (self-hosted)
  // never mounts, so the action would be disabled even for admins. Depend on hasUser so the
  // check re-evaluates once the user and their permissions load (they arrive asynchronously
  // after mount); without it a permitted non-admin would stay stuck on the initial false.
  const hasUser = ExtJS.useUser() ?? false;
  const canDeleteTag = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.TAGS.DELETE),
    [hasUser],
  );

  // Local UI state (filters, sort, etc.)
  const [searchFilter, setSearchFilter] = useState('');
  const [formatFilter, setFormatFilter] = useState('');
  const [repositoryFilter, setRepositoryFilter] = useState('');
  const [sortField, setSortField] = useState<SortField>('name');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  // Tracks a confirm click so the dialog's synchronous auto-close can be ignored
  // while the delete is in flight (see handleDeleteDialogOpenChange).
  const confirmingDeleteRef = useRef(false);

  /**
   * Navigate back to tags list.
   */
  const handleBack = useCallback(() => {
    router.stateService.go('preview.browse.tags');
  }, [router]);

  // Use the XState-powered hook
  const {
    tagDetail,
    tagLoading,
    tagError,
    components,
    componentsLoading,
    componentsError,
    hasMoreComponents,
    totalComponentCount,
    loadMoreComponents,
    retry,
    deleteTag,
    deleting,
  } = useTagDetailExtended({
    tagName,
    onDeleted: useCallback(() => {
      setDeleteDialogOpen(false);
      toast.success(TagsStrings.DETAIL.TAG_DELETED(tagName));
      handleBack();
    }, [tagName, toast, handleBack]),
    onDeleteError: useCallback(
      (message: string) => {
        toast.error(message);
      },
      [toast]
    ),
  });

  /**
   * Get unique formats from components.
   */
  const formats = useMemo(() => {
    const formatSet = new Set(components.map((c) => c.format));
    return Array.from(formatSet).sort();
  }, [components]);

  /**
   * Get unique repositories from components.
   */
  const repositories = useMemo(() => {
    const repoSet = new Set(components.map((c) => c.repository));
    return Array.from(repoSet).sort();
  }, [components]);

  /**
   * Filter and sort components.
   */
  const filteredComponents = useMemo(() => {
    let filtered = [...components];

    // Apply search filter
    if (searchFilter) {
      const search = searchFilter.toLowerCase();
      filtered = filtered.filter(
        (c) =>
          c.name.toLowerCase().includes(search) ||
          (c.group?.toLowerCase().includes(search)) ||
          c.repository.toLowerCase().includes(search)
      );
    }

    // Apply format filter
    if (formatFilter) {
      filtered = filtered.filter((c) => c.format === formatFilter);
    }

    // Apply repository filter
    if (repositoryFilter) {
      filtered = filtered.filter((c) => c.repository === repositoryFilter);
    }

    // Sort
    filtered.sort((a, b) => {
      let comparison = 0;
      switch (sortField) {
        case 'name':
          comparison = (a.name || '').localeCompare(b.name || '');
          break;
        case 'format':
          comparison = a.format.localeCompare(b.format);
          break;
        case 'version':
          comparison = (a.version || '').localeCompare(b.version || '');
          break;
        case 'repository':
          comparison = a.repository.localeCompare(b.repository);
          break;
      }
      return sortDirection === 'desc' ? -comparison : comparison;
    });

    return filtered;
  }, [components, searchFilter, formatFilter, repositoryFilter, sortField, sortDirection]);

  /**
   * Handle sort toggle.
   */
  const handleSort = useCallback(
    (field: SortField) => {
      if (sortField === field) {
        setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
      } else {
        setSortField(field);
        setSortDirection('asc');
      }
    },
    [sortField]
  );

  /**
   * Navigate to component detail.
   */
  const handleViewComponent = useCallback(
    (component: TaggedComponent) => {
      // Safety net: navigation needs both search-route permission and a component
      // name to build the gaId. Rows without both are rendered non-interactive
      // (see isRowInteractive below), so this guard should never fire in practice.
      if (!canOpenComponent || !component.name) {
        return;
      }
      // Navigate using the unified search navigation helper, which builds the
      // correct gaId and calls the component detail route. Assets are not required.
      navigateToDetail({
        id: component.id,
        name: component.name,
        format: component.format,
        repository: component.repository,
        group: component.group ?? undefined,
        // version is a required string on SearchResult; navigateToDetail treats a
        // falsy value as "no version" (the detail page then auto-selects the first).
        version: component.version ?? '',
      });
    },
    [navigateToDetail, canOpenComponent]
  );

  /**
   * Handle keyboard navigation for component rows.
   */
  const handleRowKeyDown = useCallback(
    (component: TaggedComponent) => (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        handleViewComponent(component);
      }
    },
    [handleViewComponent]
  );

  const handleExportCsv = useCallback(() => {
    exportToCsv(
      filteredComponents.map((c) => ({
        name: c.name,
        group: c.group,
        version: c.version,
        format: c.format,
        repository: c.repository,
      })),
      `${tagName}-components.csv`,
      ['name', 'group', 'version', 'format', 'repository']
    );
  }, [tagName, filteredComponents]);

  /**
   * Handle delete tag with confirmation. Fire-and-forget: the hook's onDeleted /
   * onDeleteError callbacks surface success and failure.
   */
  const handleDeleteTag = useCallback(() => {
    // ConfirmDialog fires onOpenChange(false) synchronously on confirm, before the
    // machine has entered 'deleting' (the loading prop is still false this render).
    // Flag the confirm so onOpenChange can swallow that one close and keep the
    // dialog open (with a spinner) until the delete resolves.
    confirmingDeleteRef.current = true;
    deleteTag();
  }, [deleteTag]);

  /**
   * Gate the dialog's open state: swallow the synchronous close triggered by the
   * confirm click (so the dialog stays open while deleting, and remains open on
   * error to allow a retry), but honor genuine cancels/dismissals.
   */
  const handleDeleteDialogOpenChange = useCallback((next: boolean) => {
    if (!next && confirmingDeleteRef.current) {
      confirmingDeleteRef.current = false;
      return;
    }
    setDeleteDialogOpen(next);
  }, []);

  /**
   * Render sort icon.
   */
  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null;
    return sortDirection === 'asc' ? <ChevronUp size={14} /> : <ChevronDown size={14} />;
  };

  // Loading state
  if (tagLoading) {
    return (
      <Box
        className="tag-detail-page"
        px={{ initial: '4', md: '6', lg: '6' }}
        py={{ initial: '4', md: '5', lg: '6' }}
        width="100%"
        style={{ minWidth: 0, boxSizing: 'border-box' }}
      >
        <Flex align="center" justify="center" p="9" gap="3">
          <Spinner size="3" />
          <Text>{STRINGS.loading}</Text>
        </Flex>
      </Box>
    );
  }

  // Error state
  if (tagError) {
    return (
      <Box
        className="tag-detail-page"
        px={{ initial: '4', md: '6', lg: '6' }}
        py={{ initial: '4', md: '5', lg: '6' }}
        width="100%"
        style={{ minWidth: 0, boxSizing: 'border-box' }}
      >
        <PageHeader
          title={tagName || TagsStrings.DETAIL.TAG_DETAILS}
          breadcrumbs={[
            { label: TagsStrings.DETAIL.TAGS, onClick: handleBack },
            { label: tagName || TagsStrings.DETAIL.UNKNOWN },
          ]}
        />
        <Flex direction="column" align="center" justify="center" p="9" gap="3">
          <Text color="red">{STRINGS.error}</Text>
          <Text color="gray" size="2">
            {tagError}
          </Text>
          <Button variant="soft" onClick={retry}>
            {STRINGS.retry}
          </Button>
        </Flex>
      </Box>
    );
  }

  return (
    <Box
      className="tag-detail-page"
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
      width="100%"
      style={{ minWidth: 0, boxSizing: 'border-box' }}
    >
      <PageHeader
        title={tagName || TagsStrings.DETAIL.TAG_DETAILS}
        breadcrumbs={[
          { label: TagsStrings.DETAIL.TAGS, onClick: handleBack },
          { label: tagName || TagsStrings.DETAIL.UNKNOWN },
        ]}
      />

      <ScrollArea className="tag-detail-page__content">
        {/* Tag Info Card */}
        <Card className="tag-detail-page__info-card">
          <Flex align="center" gap="4" p="4">
            <Box className="tag-detail-page__icon-wrapper">
              <Tag size={32} />
            </Box>
            <Box style={{ flex: 1 }}>
              <Heading size="6">{tagName}</Heading>
              <Flex gap="4" mt="2" wrap="wrap">
                <Flex align="center" gap="2">
                  <Calendar size={14} className="tag-detail-page__meta-icon" />
                  <Text size="2" color="gray">
                    {STRINGS.header.created}:
                  </Text>
                  <Text size="2">{formatDate(tagDetail?.firstCreated || null)}</Text>
                </Flex>
                <Flex align="center" gap="2">
                  <Clock size={14} className="tag-detail-page__meta-icon" />
                  <Text size="2" color="gray">
                    {STRINGS.header.lastUpdated}:
                  </Text>
                  <Text size="2">{formatDate(tagDetail?.lastUpdated || null)}</Text>
                </Flex>
                <Flex align="center" gap="2">
                  <Package size={14} className="tag-detail-page__meta-icon" />
                  <Text size="2" color="gray">
                    {STRINGS.header.totalComponents}:
                  </Text>
                  {/* Show an explicit "unknown" indicator when the count could not be
                      resolved, rather than falling back to the loaded page size — that
                      fallback misrepresents a 500-component tag as its first-page count. */}
                  <Badge size="1" variant="soft" color="blue">
                    {totalComponentCount !== null ? totalComponentCount : '—'}
                  </Badge>
                </Flex>
              </Flex>
            </Box>
            <Button
              variant="soft"
              color="red"
              onClick={() => setDeleteDialogOpen(true)}
              disabled={!canDeleteTag}
            >
              <Trash2 size={14} />
              {STRINGS.actions.delete}
            </Button>
          </Flex>
        </Card>

        {/* Components Section */}
        <Card className="tag-detail-page__components-card">
          <Box p="4">
            <Flex justify="between" align="center" mb="4">
              <Heading size="4">{STRINGS.components.title}</Heading>
              <Button variant="soft" onClick={handleExportCsv} disabled={filteredComponents.length === 0}>
                <Download size={14} />
                {STRINGS.components.exportCsv}
              </Button>
            </Flex>

            {/* Filters */}
            <Flex gap="3" mb="4" wrap="wrap" className="tag-detail-page__filters">
              <Box style={{ flex: 1, minWidth: '200px' }}>
                <TextField.Root
                  placeholder={STRINGS.components.searchPlaceholder}
                  value={searchFilter}
                  onChange={(e) => setSearchFilter(e.target.value)}
                >
                  <TextField.Slot>
                    <Search size={14} />
                  </TextField.Slot>
                  {searchFilter && (
                    <TextField.Slot>
                      <Button variant="ghost" size="1" onClick={() => setSearchFilter('')} data-testid="clear-search-btn">
                        <X size={14} />
                      </Button>
                    </TextField.Slot>
                  )}
                </TextField.Root>
              </Box>

              <Select.Root
                value={formatFilter || '__all__'}
                onValueChange={(v) => setFormatFilter(v === '__all__' ? '' : v)}
              >
                <Select.Trigger placeholder={STRINGS.components.allFormats} />
                <Select.Content>
                  <Select.Item value="__all__">{STRINGS.components.allFormats}</Select.Item>
                  {formats.map((format) => (
                    <Select.Item key={format} value={format}>
                      {format}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>

              <Select.Root
                value={repositoryFilter || '__all__'}
                onValueChange={(v) => setRepositoryFilter(v === '__all__' ? '' : v)}
              >
                <Select.Trigger placeholder={STRINGS.components.allRepositories} />
                <Select.Content>
                  <Select.Item value="__all__">{STRINGS.components.allRepositories}</Select.Item>
                  {repositories.map((repo) => (
                    <Select.Item key={repo} value={repo}>
                      {repo}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>
            </Flex>

            {/* Components Table */}
            {componentsLoading && components.length === 0 ? (
              <Flex align="center" justify="center" p="6" gap="2">
                <Spinner size="2" />
                <Text color="gray">{STRINGS.loadingComponents}</Text>
              </Flex>
            ) : componentsError && components.length === 0 ? (
              <Flex direction="column" align="center" justify="center" p="6" gap="3">
                <Text color="red">{STRINGS.components.error}</Text>
                <Text color="gray" size="2">
                  {componentsError}
                </Text>
                <Button variant="soft" onClick={retry}>
                  {STRINGS.components.retry}
                </Button>
              </Flex>
            ) : filteredComponents.length === 0 ? (
              <Flex direction="column" align="center" justify="center" p="6" gap="2">
                <Package size={32} color="var(--gray-9)" />
                <Text color="gray">
                  {components.length === 0 ? STRINGS.components.empty : STRINGS.components.emptyFiltered}
                </Text>
              </Flex>
            ) : (
              <Table.Root>
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeaderCell onClick={() => handleSort('name')} style={{ cursor: 'pointer' }}>
                      <Flex align="center" gap="1">
                        {STRINGS.table.component}
                        <SortIcon field="name" />
                      </Flex>
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell onClick={() => handleSort('format')} style={{ cursor: 'pointer' }}>
                      <Flex align="center" gap="1">
                        {STRINGS.table.format}
                        <SortIcon field="format" />
                      </Flex>
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell onClick={() => handleSort('version')} style={{ cursor: 'pointer' }}>
                      <Flex align="center" gap="1">
                        {STRINGS.table.version}
                        <SortIcon field="version" />
                      </Flex>
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell onClick={() => handleSort('repository')} style={{ cursor: 'pointer' }}>
                      <Flex align="center" gap="1">
                        {STRINGS.table.repository}
                        <SortIcon field="repository" />
                      </Flex>
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{STRINGS.table.actions}</Table.ColumnHeaderCell>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {filteredComponents.map((component) => {
                    // A row is interactive only when the user can open the Search
                    // component route AND the component has a name to build the gaId.
                    const isRowInteractive = canOpenComponent && Boolean(component.name);
                    return (
                    <Table.Row
                      key={component.id}
                      className={`tag-detail-page__component-row${isRowInteractive ? ' tag-detail-page__component-row--clickable' : ''}`}
                      {...(isRowInteractive && {
                        onClick: () => handleViewComponent(component),
                        onKeyDown: handleRowKeyDown(component),
                        tabIndex: 0,
                        role: 'button',
                        'aria-label': TagsStrings.DETAIL.VIEW(component.name),
                      })}
                    >
                      <Table.Cell>
                        <Flex direction="column" gap="1">
                          <Text weight="medium">{component.name}</Text>
                          {component.group && (
                            <Text size="1" color="gray" style={{ fontFamily: 'monospace' }}>
                              {component.group}
                            </Text>
                          )}
                        </Flex>
                      </Table.Cell>
                      <Table.Cell>
                        <Badge color={getFormatColor(component.format)} variant="soft">
                          {component.format}
                        </Badge>
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="2">{component.version || '-'}</Text>
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="2">{component.repository}</Text>
                      </Table.Cell>
                      <Table.Cell>
                        {isRowInteractive && (
                          <Tooltip content={STRINGS.actions.view}>
                            <Button
                              variant="ghost"
                              size="1"
                              onClick={(e) => {
                                e.stopPropagation();
                                handleViewComponent(component);
                              }}
                            >
                              <ExternalLink size={14} />
                            </Button>
                          </Tooltip>
                        )}
                      </Table.Cell>
                    </Table.Row>
                    );
                  })}
                </Table.Body>
              </Table.Root>
            )}

            {/* Load-more error: components are already loaded (the empty-state banner
                above only covers the initial-load failure), so surface the failure
                inline near the table with a retry rather than swallowing it. */}
            {componentsError && components.length > 0 && (
              <Flex direction="column" align="center" mt="4" gap="2">
                <Text color="red" size="2">
                  {STRINGS.components.loadMoreError}
                </Text>
                <Text color="gray" size="1">
                  {componentsError}
                </Text>
                <Button variant="soft" onClick={loadMoreComponents}>
                  {STRINGS.components.retry}
                </Button>
              </Flex>
            )}

            {/* Load More */}
            {hasMoreComponents && !componentsLoading && !componentsError && (
              <Flex justify="center" mt="4">
                <Button variant="soft" onClick={loadMoreComponents}>
                  {TagsStrings.DETAIL.LOAD_MORE}
                </Button>
              </Flex>
            )}
          </Box>
        </Card>
      </ScrollArea>

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        open={deleteDialogOpen}
        testId="delete-tag-dialog"
        onOpenChange={handleDeleteDialogOpenChange}
        loading={deleting}
        title={STRINGS.deleteDialog.title}
        message={STRINGS.deleteDialog.message}
        confirmLabel={STRINGS.deleteDialog.confirm}
        cancelLabel={STRINGS.deleteDialog.cancel}
        variant="danger"
        onConfirm={handleDeleteTag}
      >
        <Box mt="2">
          <Text size="2" color="red">
            {STRINGS.deleteDialog.warning}
          </Text>
        </Box>
      </ConfirmDialog>
    </Box>
  );
}

export default TagDetailPage;
