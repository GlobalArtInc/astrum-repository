

import React, { useState, useMemo, useCallback } from 'react';
import { useRouter } from '@uirouter/react';
import { Button, Flex, TextField } from '@radix-ui/themes';
import {
  HardDrive,
  Plus,
  Search,
  Cloud,
  Database,
  Server
} from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { HumanReadableUtils } from '../../../../../../interface/HumanReadableUtils';
import { Permissions } from '../../../../../../constants/Permissions';

import {
  PageHeader,
  EntityTable,
  EmptyState,
  StatusBadge,
  HelpSection,
  type TableColumn
} from '../../../../shared';

import { useBlobStoresList } from './useBlobStores';
import type { BlobStore } from './types';
import './BlobStoresList.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import { pageLanguage } from '../../../../../../interface/plural';

const STRINGS = {
  TITLE: BlobStoresStrings.LIST.BLOB_STORES,
  DESCRIPTION: BlobStoresStrings.LIST.CONFIGURE_LOCAL_CLOUD_BLOB_STORAGE,
  CREATE_BUTTON: BlobStoresStrings.LIST.CREATE_BLOB_STORE,
  FILTER_PLACEHOLDER: BlobStoresStrings.LIST.FILTER_NAME,
  LOADING: BlobStoresStrings.LIST.LOADING_BLOB_STORES,
  ERROR_TITLE: BlobStoresStrings.LIST.FAILED_LOAD_BLOB_STORES,
  EMPTY: {
    TITLE: BlobStoresStrings.LIST.NO_BLOB_STORES,
    DESCRIPTION: BlobStoresStrings.LIST.CREATE_FIRST_BLOB_STORE_CONFIGURE,
    ACTION: BlobStoresStrings.LIST.CREATE_BLOB_STORE
  },
  COLUMNS: {
    NAME: CommonStrings.LABELS.NAME,
    PATH: BlobStoresStrings.LIST.PATH,
    TYPE: CommonStrings.LABELS.TYPE,
    STATE: BlobStoresStrings.LIST.STATE,
    COUNT: BlobStoresStrings.LIST.BLOB_COUNT,
    SIZE: BlobStoresStrings.LIST.TOTAL_SIZE,
    SPACE: BlobStoresStrings.LIST.AVAILABLE_SPACE
  },
  STATUS: {
    ONLINE: BlobStoresStrings.LIST.STARTED,
    OFFLINE: BlobStoresStrings.LIST.FAILED
  },
  UNKNOWN: BlobStoresStrings.LIST.UNAVAILABLE,
  UNLIMITED: BlobStoresStrings.LIST.UNLIMITED,
  CALCULATING: BlobStoresStrings.LIST.CALCULATING,
  HELP: {
    TITLE: BlobStoresStrings.LIST.WHAT_BLOB_STORE,
    CONTENT: BlobStoresStrings.LIST.BINARY_ASSETS_YOU_DOWNLOAD_VIA,
    DOC_LABEL: CommonStrings.ACTIONS.VIEW_DOCUMENTATION
  }
};

function getTypeIcon(typeId: string): React.ReactNode {
  const iconClass = 'blob-stores-list__type-icon';
  switch (typeId?.toLowerCase()) {
    case 's3':
      return <Cloud className={`${iconClass} ${iconClass}--s3`} size={16} />;
    case 'azure':
      return <Cloud className={`${iconClass} ${iconClass}--azure`} size={16} />;
    case 'google':
      return <Cloud className={`${iconClass} ${iconClass}--google`} size={16} />;
    case 'group':
      return <Database className={`${iconClass} ${iconClass}--group`} size={16} />;
    default:
      return <Server className={`${iconClass} ${iconClass}--file`} size={16} />;
  }
}

function formatBytes(bytes: number): string {
  return HumanReadableUtils.bytesToString(bytes);
}

export default function BlobStoresList() {
  const router = useRouter();
  const { blobStores, loading, error, refetch } = useBlobStoresList();
  const [filterText, setFilterText] = useState('');
  const [sortBy, setSortBy] = useState<string>('name');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');

  const isCalculating = ExtJS.state().getValue('astrum.datastore.blobstore.metrics.calculating');
  const hasUser = ExtJS.useUser() ?? false;
  const canCreate = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.BLOB_STORES.CREATE), [hasUser]);

  // Filter data
  const filteredData = useMemo(() => {
    if (!filterText) return blobStores;
    const lowerFilter = filterText.toLowerCase();
    return blobStores.filter(store => store.name.toLowerCase().includes(lowerFilter));
  }, [blobStores, filterText]);

  // Sort data
  const sortedData = useMemo(() => {
    const sorted = [...filteredData];
    sorted.sort((a, b) => {
      const aVal = a[sortBy as keyof BlobStore];
      const bVal = b[sortBy as keyof BlobStore];

      if (typeof aVal === 'string' && typeof bVal === 'string') {
        return sortDirection === 'asc' ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal);
      }
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return sortDirection === 'asc' ? aVal - bVal : bVal - aVal;
      }
      if (typeof aVal === 'boolean' && typeof bVal === 'boolean') {
        return sortDirection === 'asc'
          ? (aVal === bVal ? 0 : aVal ? -1 : 1)
          : (aVal === bVal ? 0 : aVal ? 1 : -1);
      }
      return 0;
    });
    return sorted;
  }, [filteredData, sortBy, sortDirection]);

  const handleSort = useCallback((column: string) => {
    if (sortBy === column) {
      setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(column);
      setSortDirection('asc');
    }
  }, [sortBy]);

  const handleRowClick = useCallback((store: BlobStore) => {
    router.stateService.go('preview.admin.repository.blobstores.edit', {
      type: encodeURIComponent(store.typeId),
      name: encodeURIComponent(store.name)
    });
  }, [router]);

  const handleCreate = useCallback(() => {
    router.stateService.go('preview.admin.repository.blobstores.create');
  }, [router]);

  // Define table columns
  const columns: TableColumn<BlobStore>[] = useMemo(() => [
    {
      id: 'name',
      header: STRINGS.COLUMNS.NAME,
      accessor: (row) => (
        <Flex align="center" gap="2">
          {getTypeIcon(row.typeId)}
          <span>{row.name}</span>
        </Flex>
      ),
      sortable: true
    },
    {
      id: 'path',
      header: STRINGS.COLUMNS.PATH,
      accessor: (row) => row.path || '-',
      sortable: true
    },
    {
      id: 'type',
      header: STRINGS.COLUMNS.TYPE,
      accessor: 'type',
      sortable: true
    },
    {
      id: 'available',
      header: STRINGS.COLUMNS.STATE,
      accessor: (row) => (
        <StatusBadge
          status={row.available ? 'online' : 'offline'}
          label={row.available ? STRINGS.STATUS.ONLINE : STRINGS.STATUS.OFFLINE}
          size="small"
        />
      ),
      sortable: true
    },
    {
      id: 'blobCount',
      header: STRINGS.COLUMNS.COUNT,
      accessor: (row) => {
        if (row.unavailable) return STRINGS.UNKNOWN;
        if (isCalculating) return STRINGS.CALCULATING;
        return row.blobCount.toLocaleString(pageLanguage());
      },
      sortable: true,
      align: 'right'
    },
    {
      id: 'totalSizeInBytes',
      header: STRINGS.COLUMNS.SIZE,
      accessor: (row) => {
        if (row.unavailable) return STRINGS.UNKNOWN;
        if (isCalculating) return STRINGS.CALCULATING;
        return formatBytes(row.totalSizeInBytes);
      },
      sortable: true,
      align: 'right'
    },
    {
      id: 'availableSpaceInBytes',
      header: STRINGS.COLUMNS.SPACE,
      accessor: (row) => {
        if (row.unavailable) return STRINGS.UNKNOWN;
        if (row.unlimited) return STRINGS.UNLIMITED;
        return formatBytes(row.availableSpaceInBytes);
      },
      sortable: true,
      align: 'right'
    }
  ], [isCalculating]);

  // Empty state component
  const emptyState = (
    <EmptyState
      icon={HardDrive}
      title={STRINGS.EMPTY.TITLE}
      description={STRINGS.EMPTY.DESCRIPTION}
      action={canCreate ? {
        label: STRINGS.EMPTY.ACTION,
        onClick: handleCreate,
        icon: Plus
      } : undefined}
    />
  );

  return (
    <div className="blob-stores-list">
      <PageHeader
        icon={HardDrive}
        title={STRINGS.TITLE}
        description={STRINGS.DESCRIPTION}
        actions={
          <Button
            size="2"
            variant="solid"
            onClick={handleCreate}
            disabled={!canCreate}
          >
            <Plus size={16} />
            {STRINGS.CREATE_BUTTON}
          </Button>
        }
      >
        <TextField.Root
          placeholder={STRINGS.FILTER_PLACEHOLDER}
          value={filterText}
          onChange={(e) => setFilterText(e.target.value)}
          className="blob-stores-list__filter"
        >
          <TextField.Slot>
            <Search size={16} />
          </TextField.Slot>
        </TextField.Root>
      </PageHeader>

      <Flex className="blob-stores-list__content">
        <div className="blob-stores-list__main">
          <EntityTable<BlobStore>
            data={sortedData}
            columns={columns}
            getRowKey={(row) => row.name}
            onRowClick={handleRowClick}
            loading={loading}
            loadingMessage={STRINGS.LOADING}
            error={error || undefined}
            onRetry={refetch}
            emptyState={emptyState}
            sortBy={sortBy}
            sortDirection={sortDirection}
            onSort={handleSort}
            ariaLabel={BlobStoresStrings.LIST.BLOB_STORES_TABLE_ARIA}
          />
        </div>

        <aside className="blob-stores-list__sidebar">
          <HelpSection
            title={STRINGS.HELP.TITLE}
            content={STRINGS.HELP.CONTENT}
            docLink={{
              label: STRINGS.HELP.DOC_LABEL,
              href: 'http://links.globalart.dev/products/astrum/docs/blob-store'
            }}
          />
        </aside>
      </Flex>
    </div>
  );
}
