

import React, { useMemo, useState } from 'react';
import { Box, Flex, Text, TextField, IconButton, Tooltip } from '@radix-ui/themes';
import { Search, Key, Plus, Lock, Eye, Pencil, Trash2 } from 'lucide-react';

import {
  EntityTable,
  FilterSidebar,
  EmptyState,
  HelpSection,
  type TableColumn,
  type FilterSection,
} from '../../../../shared';

import { usePrivilegeList } from './usePrivilegeList';
import {
  Privilege,
  PrivilegesListProps,
  getPrivilegeTypeLabel,
  isReadOnlyPrivilege,
} from './types';

import './PrivilegesList.scss';
import PrivilegesStrings from '../../../../constants/pages/settings/security/PrivilegesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * PrivilegesList - Displays privileges using SUPER UI/UX standard
 * Now uses XState for state management
 *
 * Features:
 * - FilterSidebar with Type filter
 * - EntityTable with sortable columns
 * - Search functionality
 * - Loading, error, and empty states
 */
export function PrivilegesList({ onSelect, onEdit, onDelete, onCreate, canEdit = true, canDelete = true }: PrivilegesListProps) {
  // Use XState list hook
  const {
    data: privileges,
    loading,
    error,
    filters,
    setFilters,
    sortField,
    sortDirection,
    setSort,
    typeCounts,
    readOnlyCounts,
    handleFilterChange,
    handleRowClick,
    handleCreate,
  } = usePrivilegeList({ onRowClick: onSelect, onCreate });

  // Pagination state
  const [currentPage, setCurrentPage] = useState(1);
  const itemsPerPage = 25;

  // Get unique types with counts for filter sidebar
  const typeOptions = useMemo(() => {
    return typeCounts.map(([value, count]) => ({
      value,
      label: getPrivilegeTypeLabel(value),
      count,
    }));
  }, [typeCounts]);

  const handleClearFilters = () => {
    setFilters({ filter: '', typeFilter: [], readOnlyFilter: [] });
    setCurrentPage(1);
  };

  // Calculate paginated data
  const totalPages = Math.ceil(privileges.length / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const endIndex = startIndex + itemsPerPage;
  const paginatedPrivileges = privileges.slice(startIndex, endIndex);

  // Reset to page 1 when filters change
  React.useEffect(() => {
    setCurrentPage(1);
  }, []);

  // Read-only filter options (Locked = system/read-only, Unlocked = editable)
  const readOnlyOptions = useMemo(
    () => [
      { value: 'locked', label: PrivilegesStrings.LIST.LOCKED, count: readOnlyCounts.locked },
      { value: 'unlocked', label: PrivilegesStrings.LIST.UNLOCKED, count: readOnlyCounts.unlocked },
    ],
    [readOnlyCounts]
  );

  // Filter sections for sidebar
  const filterSections = useMemo<FilterSection[]>(() => {
    return [
      {
        id: 'readOnly',
        label: CommonStrings.LABELS.STATUS,
        type: 'checkbox',
        options: readOnlyOptions,
        value: filters.readOnlyFilter ?? [],
      },
      {
        id: 'type',
        label: CommonStrings.LABELS.TYPE,
        type: 'checkbox',
        options: typeOptions,
        value: filters.typeFilter,
      },
    ];
  }, [typeOptions, readOnlyOptions, filters.typeFilter, filters.readOnlyFilter]);

  // Table columns
  const columns = useMemo<TableColumn<Privilege>[]>(() => [
    {
      id: 'name',
      header: CommonStrings.LABELS.NAME,
      accessor: (priv) => (
        <Flex align="center" gap="2">
          {priv.readOnly && (
            <Lock size={14} className="privileges-list__readonly-icon" />
          )}
          <Text>{priv.name}</Text>
        </Flex>
      ),
      sortable: true,
      width: '200px',
    },
    {
      id: 'description',
      header: CommonStrings.LABELS.DESCRIPTION,
      accessor: (priv) => priv.description || '—',
      sortable: true,
      width: '300px',
    },
    {
      id: 'type',
      header: CommonStrings.LABELS.TYPE,
      accessor: (priv) => getPrivilegeTypeLabel(priv.type),
      sortable: true,
      width: '150px',
    },
    {
      id: 'permission',
      header: PrivilegesStrings.LIST.PERMISSION,
      accessor: (priv) => (
        <code className="privileges-list__permission-code">{priv.permission}</code>
      ),
      sortable: true,
      width: '250px',
    },
    {
      id: 'actions',
      header: '',
      accessor: (priv) => (
        <Flex gap="2" justify="end">
          <Tooltip content={PrivilegesStrings.LIST.VIEW_PROFILE}>
            <IconButton
              variant="ghost"
              size="1"
              onClick={(e) => { e.stopPropagation(); onSelect(priv.name); }}
              aria-label={PrivilegesStrings.LIST.VIEW_PROFILE_NAMED(priv.name)}
            >
              <Eye size={14} />
            </IconButton>
          </Tooltip>
          {canEdit && !isReadOnlyPrivilege(priv) && (
            <Tooltip content={CommonStrings.ACTIONS.EDIT}>
              <IconButton
                variant="ghost"
                size="1"
                onClick={(e) => {
                  e.stopPropagation();
                  (onEdit || onSelect)(priv.name);
                }}
                aria-label={PrivilegesStrings.LIST.EDIT_NAMED(priv.name)}
              >
                <Pencil size={14} />
              </IconButton>
            </Tooltip>
          )}
          {canDelete && onDelete && !isReadOnlyPrivilege(priv) && (
            <Tooltip content={CommonStrings.ACTIONS.DELETE}>
              <IconButton
                variant="ghost"
                size="1"
                onClick={(e) => { e.stopPropagation(); onDelete(priv.name, priv.name); }}
                aria-label={PrivilegesStrings.LIST.DELETE_NAMED(priv.name)}
              >
                <Trash2 size={14} />
              </IconButton>
            </Tooltip>
          )}
        </Flex>
      ),
      width: '100px',
    },
  ], [onSelect, onEdit, onDelete, canEdit, canDelete]);

  // Empty state
  const emptyState = useMemo(() => {
    const readOnlyFilter = filters.readOnlyFilter ?? [];
    const hasFilters = filters.filter || filters.typeFilter.length > 0 || readOnlyFilter.length > 0;

    if (hasFilters) {
      return (
        <EmptyState
          icon={Key}
          title={PrivilegesStrings.LIST.NO_MATCHING_PRIVILEGES}
          description={PrivilegesStrings.LIST.NO_PRIVILEGES_MATCH_CURRENT_FILTERS}
          action={{
            label: CommonStrings.ACTIONS.CLEAR_FILTERS,
            onClick: handleClearFilters,
          }}
        />
      );
    }

    return (
      <EmptyState
        icon={Key}
        title={PrivilegesStrings.LIST.NO_PRIVILEGES}
        description={PrivilegesStrings.LIST.CREATE_FIRST_PRIVILEGE_START_DEFINING}
        action={onCreate ? {
          label: PrivilegesStrings.LIST.CREATE_PRIVILEGE,
          onClick: handleCreate,
          icon: Plus,
        } : undefined}
        secondaryAction={{
          label: PrivilegesStrings.LIST.LEARN_MORE_ABOUT_PRIVILEGES,
          href: 'http://links.globalart.dev/products/astrum/docs/privileges',
        }}
        tip={PrivilegesStrings.LIST.PRIVILEGES_DEFINE_GRANULAR_PERMISSIONS_CAN}
      />
    );
  }, [filters, handleClearFilters, onCreate, handleCreate]);

  return (
    <Flex className="privileges-list" gap="4" data-testid="privileges-list">
      {/* Filter Sidebar */}
      <FilterSidebar
        sections={filterSections}
        onFilterChange={handleFilterChange}
        onClear={handleClearFilters}
        disabled={loading}
        className="privileges-list__sidebar"
      />

      {/* Main Content */}
      <Box className="privileges-list__main">
        {/* Search Bar */}
        <Box className="privileges-list__search-container">
          <TextField.Root
            placeholder={PrivilegesStrings.LIST.SEARCH_PRIVILEGES_NAME_DESCRIPTION_PERMISSION_PLACEHOLDER}
            value={filters.filter}
            onChange={(e) => setFilters({ filter: e.target.value })}
            className="privileges-list__search-input"
            data-testid="privileges-search"
            data-analytics-id="as-privilege-filter"
          >
            <TextField.Slot>
              <Search size={16} />
            </TextField.Slot>
          </TextField.Root>
        </Box>

        {/* Table */}
        <EntityTable<Privilege>
          data={paginatedPrivileges}
          columns={columns}
          getRowKey={(priv) => priv.id || priv.name}
          onRowClick={handleRowClick}
          loading={loading}
          loadingMessage={PrivilegesStrings.LIST.LOADING_PRIVILEGES}
          error={error || undefined}
          onRetry={() => window.location.reload()}
          emptyState={emptyState}
          sortBy={sortField}
          sortDirection={sortDirection === 'asc' ? 'asc' : sortDirection === 'desc' ? 'desc' : undefined}
          onSort={setSort}
          showRowArrow={false}
          clickable={true}
          ariaLabel={PrivilegesStrings.LIST.PRIVILEGES_LIST_ARIA}
          className="privileges-list__table"
        />

        {/* Pagination & Summary */}
        {!(loading || error ) && privileges.length > 0 && (
          <Flex justify="between" align="center" className="privileges-list__footer">
            <Text size="2" color="gray">
              {PrivilegesStrings.LIST.SHOWING_PRIVILEGES(startIndex + 1, Math.min(endIndex, privileges.length), privileges.length)}
            </Text>
            {totalPages > 1 && (
              <Flex gap="2" align="center">
                <IconButton
                  variant="soft"
                  size="1"
                  disabled={currentPage === 1}
                  onClick={() => setCurrentPage(currentPage - 1)}
                  aria-label={PrivilegesStrings.LIST.PREVIOUS_PAGE_ARIA}
                >
                  ←
                </IconButton>
                <Text size="2" color="gray">
                  {PrivilegesStrings.LIST.PAGE_OF(currentPage, totalPages)}
                </Text>
                <IconButton
                  variant="soft"
                  size="1"
                  disabled={currentPage === totalPages}
                  onClick={() => setCurrentPage(currentPage + 1)}
                  aria-label={PrivilegesStrings.LIST.NEXT_PAGE_ARIA}
                >
                  →
                </IconButton>
              </Flex>
            )}
          </Flex>
        )}

        {/* Help Section */}
        <HelpSection
          title={PrivilegesStrings.LIST.WHAT_PRIVILEGE}
          content={PrivilegesStrings.LIST.PRIVILEGE_GRANULAR_PERMISSION_CONTROLS_ACCESS}
          docLink={{
            label: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
            href: 'http://links.globalart.dev/products/astrum/docs/privileges',
          }}
          className="privileges-list__help"
        />
      </Box>
    </Flex>
  );
}

export default PrivilegesList;
