

import React, { useState, useEffect, useCallback, useMemo, } from 'react';
import { Box, Flex, Text, Button, TextField, } from '@radix-ui/themes';
import {
  HardDrive,
  Plus,
  Search,
  AlertCircle,
  Trash2,
  AlertTriangle,
  RefreshCw,
  Cloud,
  Database,
  Server,
} from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { HumanReadableUtils } from '../../../../../../interface/HumanReadableUtils';
import { Permissions } from '../../../../../../constants/Permissions';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsAlert,
  SettingsButton,
} from '../../../../shared/form';
import {
  PageHeader,
  EntityTable,
  EmptyState,
  StatusBadge,
  HelpSection,
  useToast,
  type TableColumn,
} from '../../../../shared';
import { clearDirtyState } from '../../../../shared';
import {
  useBlobStoresList,
  useBlobStore,
  useBlobStoreTypes,
  useBlobStorePromote,
} from './useBlobStores';
import FileBlobStoreSettings from './FileBlobStoreSettings';
import S3BlobStoreSettings from './S3BlobStoreSettings';
import AzureBlobStoreSettings from './AzureBlobStoreSettings';
import GoogleBlobStoreSettings from './GoogleBlobStoreSettings';
import GroupBlobStoreSettings from './GroupBlobStoreSettings';
import ConvertToGroupModal from './ConvertToGroupModal';
import { BlobStoreWizardCreate } from './BlobStoreWizardCreate';
import { DangerousEditConfirmDialog } from './DangerousEditConfirmDialog';
import { hasDangerousFieldChanges, getDangerousFieldsChanged } from './dangerousFields';
import type { BlobStore, BlobStoreFormData, SoftQuota } from './types';
import './BlobStoresPage.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';
import CommonStrings from '../../../../constants/CommonStrings';

// Base path for blob stores URLs
const BASE_PATH = 'preview/admin/repository/blobstores';

/**
 * URL-based routing patterns:
 * - /blobstores                 → List page
 * - /blobstores/create          → Create form (type selection)
 * - /blobstores/{type}/{name}   → Edit form
 */
type ViewMode = 'list' | 'create' | 'detail';

interface RouteState {
  viewMode: ViewMode;
  blobStoreType: string | null;
  blobStoreName: string | null;
}

const STRINGS = {
  TITLE: BlobStoresStrings.PAGE.BLOB_STORES,
  DESCRIPTION: BlobStoresStrings.PAGE.CONFIGURE_LOCAL_CLOUD_BLOB_STORAGE,
  CREATE_BUTTON: BlobStoresStrings.PAGE.CREATE_BLOB_STORE,
  FILTER_PLACEHOLDER: BlobStoresStrings.PAGE.FILTER_NAME,
  LOADING: BlobStoresStrings.PAGE.LOADING_BLOB_STORES,
  ERROR_TITLE: BlobStoresStrings.PAGE.FAILED_LOAD_BLOB_STORES,
  EMPTY: {
    TITLE: BlobStoresStrings.PAGE.NO_BLOB_STORES,
    DESCRIPTION: BlobStoresStrings.PAGE.CREATE_FIRST_BLOB_STORE_CONFIGURE,
    ACTION: BlobStoresStrings.PAGE.CREATE_BLOB_STORE,
  },
  COLUMNS: {
    NAME: CommonStrings.LABELS.NAME,
    PATH: BlobStoresStrings.PAGE.PATH,
    TYPE: CommonStrings.LABELS.TYPE,
    STATE: BlobStoresStrings.PAGE.STATE,
    COUNT: BlobStoresStrings.PAGE.BLOB_COUNT,
    SIZE: BlobStoresStrings.PAGE.TOTAL_SIZE,
    SPACE: BlobStoresStrings.PAGE.AVAILABLE_SPACE,
  },
  STATUS: {
    ONLINE: BlobStoresStrings.PAGE.STARTED,
    OFFLINE: BlobStoresStrings.PAGE.FAILED,
  },
  UNKNOWN: BlobStoresStrings.PAGE.UNAVAILABLE,
  UNLIMITED: BlobStoresStrings.PAGE.UNLIMITED,
  CALCULATING: BlobStoresStrings.PAGE.CALCULATING,
  HELP: {
    TITLE: BlobStoresStrings.PAGE.WHAT_BLOB_STORE,
    CONTENT:
      BlobStoresStrings.PAGE.BINARY_ASSETS_YOU_DOWNLOAD_VIA,
    DOC_LABEL: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
  },
  CREATE_TITLE: BlobStoresStrings.PAGE.CREATE_BLOB_STORE,
  EDIT_TITLE: (name: string) => BlobStoresStrings.PAGE.EDIT(name),
  EDIT_DESCRIPTION: (type: string) => BlobStoresStrings.PAGE.BLOB_STORE(type),
  EDIT_WARNING:
    BlobStoresStrings.PAGE.UPDATING_BLOB_STORE_CONFIGURATION_CAUSE,
  CONVERT_TO_GROUP_BUTTON: BlobStoresStrings.PAGE.CONVERT_GROUP,
  TYPE: {
    label: CommonStrings.LABELS.TYPE,
    sublabel: BlobStoresStrings.PAGE.SELECT_TYPE_BLOB_STORE,
  },
  NAME: {
    label: CommonStrings.LABELS.NAME,
    placeholder: BlobStoresStrings.PAGE.ENTER_BLOB_STORE_NAME,
  },
  SOFT_QUOTA: {
    title: BlobStoresStrings.PAGE.SOFT_QUOTA,
    description: BlobStoresStrings.PAGE.RAISES_ALERT_WHEN_BLOB_STORE,
    ENABLED: BlobStoresStrings.PAGE.ENABLE_SOFT_QUOTA,
    TYPE: {
      label: BlobStoresStrings.PAGE.CONSTRAINT_TYPE,
      placeholder: BlobStoresStrings.PAGE.SELECT_CONSTRAINT_TYPE,
    },
    LIMIT: {
      label: BlobStoresStrings.PAGE.CONSTRAINT_LIMIT_MB,
      placeholder: BlobStoresStrings.PAGE.ENTER_LIMIT_MB,
    },
  },
  DELETE: {
    button: CommonStrings.ACTIONS.DELETE,
    tooltip: (repositoryUsage: number, blobStoreUsage: number, hasPermission: boolean) => {
      if (!hasPermission) {
        return BlobStoresStrings.PAGE.YOU_DONT_HAVE_PERMISSION_DELETE;
      }
      return BlobStoresStrings.PAGE.BLOB_STORE_USE_REPOSITORIES_BLOB(repositoryUsage, blobStoreUsage);
    },
  },
  CONFIRM_DELETE: {
    title: BlobStoresStrings.PAGE.DELETE_BLOB_STORE,
    message: BlobStoresStrings.PAGE.YOU_SURE_YOU_WANT_DELETE,
    confirm: CommonStrings.ACTIONS.DELETE,
    cancel: CommonStrings.ACTIONS.CANCEL,
  },
  SAVE_SUCCESS: (name: string, isCreate: boolean) =>
    isCreate ? BlobStoresStrings.PAGE.BLOB_STORE_CREATED_SUCCESSFULLY(name) : BlobStoresStrings.PAGE.BLOB_STORE_UPDATED_SUCCESSFULLY(name),
  DELETE_SUCCESS: (name: string) => BlobStoresStrings.PAGE.BLOB_STORE_DELETED_SUCCESSFULLY(name),
};

const SPACE_USED_QUOTA_ID = 'spaceUsedQuota';

/**
 * Parse the URL hash to determine the current route state
 */
function parseRoute(hash: string): RouteState {
  // Remove leading # and any query params (like ?debug)
  const cleanHash = hash.replace(/^#/, '').replace(/\?.*$/, '');
  const parts = cleanHash.split('/');

  // Find the blobstores segment
  const blobstoresIndex = parts.indexOf('blobstores');
  if (blobstoresIndex === -1) {
    return { viewMode: 'list', blobStoreType: null, blobStoreName: null };
  }

  const pathAfterBlobstores = parts.slice(blobstoresIndex + 1);

  // /blobstores → list
  if (pathAfterBlobstores.length === 0 || pathAfterBlobstores[0] === '') {
    return { viewMode: 'list', blobStoreType: null, blobStoreName: null };
  }

  // /blobstores/create → create form
  if (pathAfterBlobstores[0] === 'create') {
    return { viewMode: 'create', blobStoreType: null, blobStoreName: null };
  }

  // /blobstores/{type}/{name} → detail/edit
  if (pathAfterBlobstores.length >= 2) {
    return {
      viewMode: 'detail',
      blobStoreType: decodeURIComponent(pathAfterBlobstores[0]),
      blobStoreName: decodeURIComponent(pathAfterBlobstores[1]),
    };
  }

  return { viewMode: 'list', blobStoreType: null, blobStoreName: null };
}

/**
 * Navigate to a new route by updating the URL hash
 */
function navigateTo(path: string) {
  window.location.hash = path;
}

function getTypeIcon(typeId: string): React.ReactNode {
  const iconClass = 'blob-stores-page__type-icon';
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

/**
 * BlobStoresPage - Main Blob Stores management page for Preview UI
 *
 * Uses URL-based routing for testability:
 * - Direct URL access to any view
 * - Browser back/forward support
 * - Bookmarkable URLs
 *
 * URL patterns:
 * - #preview/admin/repository/blobstores                → List
 * - #preview/admin/repository/blobstores/create         → Create form
 * - #preview/admin/repository/blobstores/{type}/{name}  → Edit form
 */
export default function BlobStoresPage() {
  const [routeState, setRouteState] = useState<RouteState>(() => parseRoute(window.location.hash));
  const [filterText, setFilterText] = useState('');
  const [sortBy, setSortBy] = useState<string>('name');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [error, setError] = useState<string | null>(null);
  const [_refreshKey, setRefreshKey] = useState(0);

  // Toast notifications (app-level provider)
  const toast = useToast();

  // Form state
  const [formData, setFormData] = useState<BlobStoreFormData>({ name: '' });
  const [selectedType, setSelectedType] = useState<string>('');
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [showConvertModal, setShowConvertModal] = useState(false);
  const [showDangerousEditDialog, setShowDangerousEditDialog] = useState(false);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});

  // Track original data for dirty state detection
  const [baseData, setBaseData] = useState<string>('');

  // Data hooks
  const { blobStores, loading: listLoading, error: listError, refetch } = useBlobStoresList();
  const { types, quotaTypes, loading: typesLoading, error: typesError } = useBlobStoreTypes();
  const {
    blobStore,
    blobStoreUsage,
    repositoryUsage,
    loading: blobStoreLoading,
    save,
    remove,
  } = useBlobStore(routeState.blobStoreName || undefined, routeState.blobStoreType || undefined);
  const { promoting, promote } = useBlobStorePromote();

  // Permissions
  const isCalculating =
    ExtJS.state().getValue('astrum.datastore.blobstore.metrics.calculating') || false;
  const hasUser = ExtJS.useUser() ?? false;
  const canCreate = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.BLOB_STORES.CREATE),
    [hasUser]
  );
  const hasUpdatePermissions = ExtJS.checkPermission(Permissions.BLOB_STORES.UPDATE);
  const hasDeletePermission = ExtJS.checkPermission(Permissions.BLOB_STORES.DELETE);

  // Listen for hash changes (browser back/forward, direct URL navigation)
  useEffect(() => {
    const handleHashChange = () => {
      const newState = parseRoute(window.location.hash);
      setRouteState(newState);
      setError(null);
      setValidationErrors({});
    };

    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  // Initialize form data when blob store loads (edit mode)
  useEffect(() => {
    if (blobStore && routeState.viewMode === 'detail') {
      // The REST API GET endpoints (file, s3, azure, google) do NOT return `name` or `type` —
      // both are implied by the URL path segments. We must synthesize them from the route state
      // so that:
      //   1. validate() passes (it checks formData.name — would silently block save otherwise)
      //   2. The dirty comparison is accurate (formData and baseData must have the same shape)
      const typeValue =
        blobStore.type ||
        (blobStore as Record<string, unknown>).typeId as string ||
        routeState.blobStoreType ||
        '';
      const normalized: BlobStoreFormData = {
        ...blobStore,
        name: blobStore.name || routeState.blobStoreName || '',
        type: typeValue,
      };
      setFormData(normalized);
      // baseData is set from the same normalized object so JSON.stringify produces
      // identical output and isDirty starts as false on initial load.
      setBaseData(JSON.stringify(normalized));
      setSelectedType(typeValue);
    } else if (routeState.viewMode === 'create') {
      setFormData({ name: '' });
      setSelectedType('');
      setBaseData('');
    }
  }, [blobStore, routeState.viewMode, routeState.blobStoreType, routeState.blobStoreName]);

  // Calculate dirty state
  const isDirty = useMemo(() => {
    // If baseData is empty, it means we are in initial create state
    const originalData = baseData || JSON.stringify({ name: '' });
    return JSON.stringify(formData) !== originalData;
  }, [formData, baseData]);

  // Filter data
  const filteredData = useMemo(() => {
    if (!filterText) return blobStores;
    const lowerFilter = filterText.toLowerCase();
    return blobStores.filter((store) => store.name.toLowerCase().includes(lowerFilter));
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
          ? aVal === bVal
            ? 0
            : aVal
              ? -1
              : 1
          : aVal === bVal
            ? 0
            : aVal
              ? 1
              : -1;
      }
      return 0;
    });
    return sorted;
  }, [filteredData, sortBy, sortDirection]);

  // Navigation handlers
  const handleSort = useCallback(
    (column: string) => {
      if (sortBy === column) {
        setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
      } else {
        setSortBy(column);
        setSortDirection('asc');
      }
    },
    [sortBy]
  );

  const handleRowClick = useCallback((store: BlobStore) => {
    navigateTo(
      `${BASE_PATH}/${encodeURIComponent(store.typeId)}/${encodeURIComponent(store.name)}`
    );
  }, []);

  const handleCreate = useCallback(() => {
    navigateTo(`${BASE_PATH}/create`);
  }, []);

  const handleBack = useCallback(() => {
    navigateTo(BASE_PATH);
  }, []);

  // Form handlers
  const updateField = useCallback((field: string, value: unknown) => {
    setFormData((prev) => ({
      ...prev,
      [field]: value,
    }));
    setValidationErrors((prev) => {
      const next = { ...prev };
      delete next[field];
      return next;
    });
  }, []);

  const updateNestedField = useCallback((path: string, value: unknown) => {
    setFormData((prev) => {
      const parts = path.split('.');
      const result = { ...prev };
      let current: Record<string, unknown> = result;

      for (let i = 0; i < parts.length - 1; i++) {
        const part = parts[i];
        current[part] = { ...((current[part] as Record<string, unknown>) || {}) };
        current = current[part] as Record<string, unknown>;
      }

      current[parts[parts.length - 1]] = value;
      return result;
    });
  }, []);

  // Soft quota handling
  const handleSoftQuotaToggle = useCallback(
    (enabled: boolean) => {
      const currentQuota = formData.softQuota || { enabled: false };
      const newQuota: SoftQuota = {
        ...currentQuota,
        enabled,
        type: enabled ? currentQuota.type || SPACE_USED_QUOTA_ID : undefined,
        limit: enabled ? currentQuota.limit : undefined,
      };
      updateField('softQuota', newQuota);
    },
    [formData.softQuota, updateField]
  );

  const handleSoftQuotaTypeChange = useCallback(
    (type: string) => {
      updateField('softQuota', {
        ...formData.softQuota,
        type,
      });
    },
    [formData.softQuota, updateField]
  );

  const handleSoftQuotaLimitChange = useCallback(
    (limit: string) => {
      updateField('softQuota', {
        ...formData.softQuota,
        limit: limit ? parseInt(limit, 10) : undefined,
      });
    },
    [formData.softQuota, updateField]
  );

  // Type selection
  const handleTypeChange = useCallback(
    (typeId: string) => {
      setSelectedType(typeId);
      const lowerTypeId = typeId.toLowerCase();

      if (lowerTypeId === 's3') {
        setFormData((prev) => ({
          ...prev,
          type: typeId,
          bucketConfiguration: {
            bucket: { region: 'DEFAULT', name: '', prefix: '' },
            bucketSecurity: {},
            encryption: null,
            advancedBucketConnection: {},
            failoverBuckets: [],
            activeRegion: null,
          },
        }));
      } else if (lowerTypeId === 'azure') {
        setFormData((prev) => ({
          ...prev,
          type: typeId,
          bucketConfiguration: {
            accountName: '',
            containerName: '',
            authentication: { authenticationMethod: 'ENVIRONMENTVARIABLE' },
          },
        }));
      } else if (lowerTypeId === 'google') {
        setFormData((prev) => ({
          ...prev,
          type: typeId,
          bucketConfiguration: {
            bucket: { name: '', prefix: '' },
            bucketSecurity: {},
          },
        }));
      } else if (lowerTypeId === 'file') {
        setFormData((prev) => ({
          ...prev,
          type: typeId,
          path: '',
        }));
      } else if (lowerTypeId === 'group') {
        setFormData((prev) => ({
          ...prev,
          type: typeId,
          members: [],
          fillPolicy: '',
        }));
      } else {
        updateField('type', typeId);
      }
    },
    [updateField]
  );

  // Validation
  const validate = useCallback((): boolean => {
    const errors: Record<string, string> = {};

    if (!formData.name?.trim()) {
      errors.name = CommonStrings.MESSAGES.NAME_REQUIRED;
    } else if (!/^[a-zA-Z0-9_-]+$/.test(formData.name)) {
      errors.name = BlobStoresStrings.PAGE.NAME_CAN_ONLY_CONTAIN_LETTERS;
    }

    if (!selectedType && routeState.viewMode === 'create') {
      errors.type = BlobStoresStrings.PAGE.TYPE_REQUIRED;
    }

    const typeToCheck = (
      selectedType ||
      (routeState.viewMode === 'detail' ? formData.type : '')
    )?.toLowerCase();

    if (typeToCheck === 's3') {
      const bucket = (formData.bucketConfiguration as Record<string, unknown>)?.bucket as
        | Record<string, string>
        | undefined;
      if (!bucket?.name?.trim()) {
        errors['bucketConfiguration.bucket.name'] = BlobStoresStrings.PAGE.BUCKET_NAME_REQUIRED_S3;
      }
    }

    if (typeToCheck === 'azure') {
      const config = formData.bucketConfiguration as Record<string, string> | undefined;
      if (!config?.accountName?.trim()) {
        errors['bucketConfiguration.accountName'] = BlobStoresStrings.PAGE.ACCOUNT_NAME_REQUIRED_AZURE;
      }
      if (!config?.containerName?.trim()) {
        errors['bucketConfiguration.containerName'] = BlobStoresStrings.PAGE.CONTAINER_NAME_REQUIRED_AZURE;
      }
    }

    if (typeToCheck === 'google') {
      const bucket = (formData.bucketConfiguration as Record<string, unknown>)?.bucket as
        | Record<string, string>
        | undefined;
      if (!bucket?.name?.trim()) {
        errors['bucketConfiguration.bucket.name'] = BlobStoresStrings.PAGE.BUCKET_NAME_REQUIRED_GOOGLE_CLOUD;
      }
    }

    if (typeToCheck === 'file') {
      if (!formData.path?.trim()) {
        errors.path = BlobStoresStrings.PAGE.PATH_REQUIRED_FILE_BLOB_STORES;
      }
    }

    if (typeToCheck === 'group') {
      if (!formData.members || formData.members.length === 0) {
        errors.members = BlobStoresStrings.PAGE.LEAST_ONE_MEMBER_BLOB_STORE;
      }
      if (!formData.fillPolicy) {
        errors.fillPolicy = BlobStoresStrings.PAGE.FILL_POLICY_REQUIRED_GROUP_BLOB;
      }
    }

    if (formData.softQuota?.enabled) {
      if (!formData.softQuota.type) {
        errors['softQuota.type'] = BlobStoresStrings.PAGE.CONSTRAINT_TYPE_REQUIRED_WHEN_SOFT;
      }
      if (!formData.softQuota.limit || formData.softQuota.limit <= 0) {
        errors['softQuota.limit'] = BlobStoresStrings.PAGE.CONSTRAINT_LIMIT_MUST_GREATER_THAN;
      }
    }

    setValidationErrors(errors);
    return Object.keys(errors).length === 0;
  }, [formData, selectedType, routeState.viewMode]);

  // Dangerous edit detection - parse pristine data from baseData
  const pristineData = useMemo(() => {
    if (!baseData) return undefined;
    try {
      return JSON.parse(baseData) as BlobStoreFormData;
    } catch {
      return undefined;
    }
  }, [baseData]);

  const dangerousFieldsChanged = useMemo(() => {
    if (routeState.viewMode !== 'detail' || !pristineData) return [];
    return getDangerousFieldsChanged(pristineData, formData, selectedType);
  }, [routeState.viewMode, pristineData, formData, selectedType]);

  // The actual save logic, called directly or after dangerous edit confirmation
  const performSave = useCallback(async () => {
    setSaving(true);
    setError(null);

    const blobStoreName = formData.name || routeState.blobStoreName || '';

    try {
      const typeToSave = selectedType || formData.type || routeState.blobStoreType || '';
      if (!typeToSave) {
        toast.error(BlobStoresStrings.PAGE.BLOB_STORE_TYPE_REQUIRED);
        setSaving(false);
        return;
      }

      const dataToSave = { ...formData, type: typeToSave };
      await save(dataToSave);

      setBaseData(JSON.stringify(dataToSave));
      clearDirtyState(`blob-store-form-${routeState.blobStoreName || 'new'}`);

      setRefreshKey((k) => k + 1);
      navigateTo(BASE_PATH);
      toast.success(STRINGS.SAVE_SUCCESS(blobStoreName, routeState.viewMode === 'create'));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : BlobStoresStrings.PAGE.FAILED_SAVE_BLOB_STORE);
      throw err;
    } finally {
      setSaving(false);
    }
  }, [formData, selectedType, save, routeState.blobStoreName, routeState.viewMode, routeState.blobStoreType, toast]);

  // Save with dangerous field gate
  const handleSave = useCallback(() => {
    if (!validate()) return;

    const isEdit = routeState.viewMode === 'detail';
    if (isEdit && pristineData && hasDangerousFieldChanges(pristineData, formData, selectedType)) {
      setShowDangerousEditDialog(true);
    } else {
      performSave();
    }
  }, [validate, routeState.viewMode, pristineData, formData, selectedType, performSave]);

  const handleConfirmDangerousEdit = useCallback(() => {
    setShowDangerousEditDialog(false);
    performSave();
  }, [performSave]);

  // Delete
  const handleDelete = useCallback(async () => {
    setDeleting(true);
    setError(null);

    const blobStoreName = routeState.blobStoreName || '';

    try {
      await remove();
      toast.success(STRINGS.DELETE_SUCCESS(blobStoreName));
      setRefreshKey((k) => k + 1);
      navigateTo(BASE_PATH);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : BlobStoresStrings.PAGE.FAILED_DELETE_BLOB_STORE);
    } finally {
      setDeleting(false);
      setShowDeleteConfirm(false);
    }
  }, [remove, routeState.blobStoreName, toast]);

  // Convert to group
  const handleConvertToGroup = useCallback(
    async (newGroupName: string) => {
      if (!routeState.blobStoreName) return;

      try {
        await promote(routeState.blobStoreName, newGroupName);
        setShowConvertModal(false);
        toast.success(BlobStoresStrings.PAGE.BLOB_STORE_CONVERTED_GROUP_SUCCESSFULLY(routeState.blobStoreName, newGroupName));
        setRefreshKey((k) => k + 1);
        navigateTo(BASE_PATH);
      } catch (err) {
        toast.error(err instanceof Error ? err.message : BlobStoresStrings.PAGE.FAILED_CONVERT_GROUP);
      }
    },
    [routeState.blobStoreName, promote, toast]
  );

  // Get current type configuration
  const currentType = types.find(
    (t) => t.id === selectedType || t.id === routeState.blobStoreType
  );
  const canDeleteBlobStore =
    (blobStoreUsage ?? 0) === 0 &&
    (repositoryUsage ?? 0) === 0 &&
    hasDeletePermission;
  const canConvertToGroup =
    routeState.viewMode === 'detail' &&
    currentType?.id !== 'group' &&
    types.some((t) => t.id === 'group');

  // Define table columns
  const columns: TableColumn<BlobStore>[] = useMemo(
    () => [
      {
        id: 'name',
        header: STRINGS.COLUMNS.NAME,
        accessor: (row) => (
          <Flex align="center" gap="2">
            {getTypeIcon(row.typeId)}
            <span>{row.name}</span>
          </Flex>
        ),
        sortable: true,
      },
      {
        id: 'path',
        header: STRINGS.COLUMNS.PATH,
        accessor: (row) => row.path || '-',
        sortable: true,
      },
      {
        id: 'type',
        header: STRINGS.COLUMNS.TYPE,
        accessor: 'type',
        sortable: true,
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
        sortable: true,
      },
      {
        id: 'blobCount',
        header: STRINGS.COLUMNS.COUNT,
        accessor: (row) => {
          if (row.unavailable) return STRINGS.UNKNOWN;
          if (isCalculating) return STRINGS.CALCULATING;
          return row.blobCount.toLocaleString();
        },
        sortable: true,
        align: 'right',
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
        align: 'right',
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
        align: 'right',
      },
    ],
    [isCalculating]
  );

  // Empty state component
  const emptyState = (
    <EmptyState
      icon={HardDrive}
      title={STRINGS.EMPTY.TITLE}
      description={STRINGS.EMPTY.DESCRIPTION}
      action={
        canCreate
          ? {
              label: STRINGS.EMPTY.ACTION,
              onClick: handleCreate,
              icon: Plus,
            }
          : undefined
      }
    />
  );

  // Loading state for form
  if (
    (routeState.viewMode === 'detail' && blobStoreLoading) ||
    (routeState.viewMode === 'create' && typesLoading)
  ) {
    return (
      <Box className="blob-stores-page" data-testid="blob-stores-page" data-view={routeState.viewMode}>
        <Box className="blob-stores-page__loading">
          <Text>{CommonStrings.MESSAGES.LOADING}</Text>
        </Box>
      </Box>
    );
  }

  // Render list view
  const renderListView = () => (
    <>
      <PageHeader
        icon={HardDrive}
        title={STRINGS.TITLE}
        description={STRINGS.DESCRIPTION}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
          { label: BlobStoresStrings.PAGE.BLOB_STORES }
        ]}
        actions={
          <Button
            size="2"
            variant="solid"
            onClick={handleCreate}
            disabled={!canCreate}
            data-testid="create-blob-store-button"
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
          className="blob-stores-page__filter"
          data-testid="filter-input"
        >
          <TextField.Slot>
            <Search size={16} />
          </TextField.Slot>
        </TextField.Root>
      </PageHeader>

      {/* Error Alert */}
      {listError && (
        <Box className="blob-stores-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {listError}
          </SettingsAlert>
        </Box>
      )}

      <Flex className="blob-stores-page__content">
        <div className="blob-stores-page__main">
          <EntityTable<BlobStore>
            data={sortedData}
            columns={columns}
            getRowKey={(row) => row.name}
            onRowClick={handleRowClick}
            loading={listLoading}
            loadingMessage={STRINGS.LOADING}
            error={listError || undefined}
            onRetry={refetch}
            emptyState={emptyState}
            sortBy={sortBy}
            sortDirection={sortDirection}
            onSort={handleSort}
            ariaLabel={BlobStoresStrings.PAGE.BLOB_STORES_TABLE_ARIA}
          />
        </div>

        <aside className="blob-stores-page__sidebar">
          <HelpSection
            title={STRINGS.HELP.TITLE}
            content={STRINGS.HELP.CONTENT}
            docLink={{
              label: STRINGS.HELP.DOC_LABEL,
              href: 'https://help.globalart.dev/en/blob-stores.html',
            }}
          />
        </aside>
      </Flex>
    </>
  );

  // Render form view (create/edit)
  const renderFormView = () => {
    const isEdit = routeState.viewMode === 'detail';
    const title = isEdit
      ? STRINGS.EDIT_TITLE(routeState.blobStoreName!)
      : STRINGS.CREATE_TITLE;

    const formBreadcrumbs = [
      { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
      { label: BlobStoresStrings.PAGE.BLOB_STORES, onClick: handleBack },
      { label: isEdit ? (routeState.blobStoreName || CommonStrings.MESSAGES.LOADING) : CommonStrings.ACTIONS.CREATE },
    ];

    return (
      <Box className="blob-stores-page__form-container">
        <PageHeader
          title={title}
          description={isEdit && currentType ? STRINGS.EDIT_DESCRIPTION(currentType.name) : undefined}
          breadcrumbs={formBreadcrumbs}
        />
        <SettingsForm
          onSave={hasUpdatePermissions ? handleSave : undefined}
          onCancel={handleBack}
          saving={saving}
          dirty={isDirty}
          testId="blob-store-form"
          data-loading={blobStoreLoading || typesLoading ? 'true' : 'false'}
          data-dirty={isDirty ? 'true' : 'false'}
          data-valid={Object.keys(validationErrors).length === 0 ? 'true' : 'false'}
          data-mode={isEdit ? 'edit' : 'create'}
          headerActions={canConvertToGroup && hasUpdatePermissions && (
                <SettingsButton
                  variant="secondary"
                  onClick={() => setShowConvertModal(true)}
                  disabled={promoting}
                  icon={RefreshCw}
                  testId="convert-to-group-button"
                >
                  {STRINGS.CONVERT_TO_GROUP_BUTTON}
                </SettingsButton>
              )}
          footerExtra={
            isEdit && (
              <SettingsButton
                variant="danger"
                onClick={() => setShowDeleteConfirm(true)}
                disabled={!canDeleteBlobStore || deleting}
                icon={Trash2}
                title={
                  !canDeleteBlobStore
                    ? STRINGS.DELETE.tooltip(repositoryUsage, blobStoreUsage, hasDeletePermission)
                    : undefined
                }
                testId="delete-button"
              >
                {STRINGS.DELETE.button}
              </SettingsButton>
            )
          }
        >
          {error && (
            <SettingsAlert variant="error" icon={<AlertCircle size={16} />}>
              {error}
            </SettingsAlert>
          )}

          {typesError && !isEdit && (
            <SettingsAlert variant="error" icon={<AlertCircle size={16} />}>
              {BlobStoresStrings.PAGE.FAILED_LOAD_BLOB_STORE_TYPES(typesError)}
            </SettingsAlert>
          )}

          {isEdit && hasUpdatePermissions && (
            <SettingsAlert variant="warning" icon={<AlertTriangle size={16} />}>
              {STRINGS.EDIT_WARNING}
            </SettingsAlert>
          )}

          {/* Type Selection - only for create */}
          {!isEdit && (
            <SettingsFormSection title={BlobStoresStrings.PAGE.BLOB_STORE_TYPE}>
              {typesLoading ? (
                <Box style={{ padding: 'var(--space-4)' }}>
                  <Text size="2">{BlobStoresStrings.PAGE.LOADING_BLOB_STORE_TYPES}</Text>
                </Box>
              ) : types.length === 0 ? (
                <SettingsAlert variant="error" icon={<AlertCircle size={16} />}>
                  {BlobStoresStrings.PAGE.NO_BLOB_STORE_TYPES_AVAILABLE}
                </SettingsAlert>
              ) : (
                <SettingsSelect
                  name="blobstore-type"
                  label={STRINGS.TYPE.label}
                  helpText={STRINGS.TYPE.sublabel}
                  value={selectedType}
                  onChange={handleTypeChange}
                  options={[
                    { value: '', label: BlobStoresStrings.PAGE.SELECT_TYPE },
                    ...types.map((t) => ({ value: t.id, label: t.name })),
                  ]}
                  required
                  error={validationErrors.type}
                  disabled={!hasUpdatePermissions || typesLoading}
                />
              )}
            </SettingsFormSection>
          )}

          {/* Name - only for create */}
          {!isEdit && selectedType && (
            <SettingsFormSection title={BlobStoresStrings.PAGE.BASIC_CONFIGURATION}>
              <SettingsTextInput
                name="blobstore-name"
                label={STRINGS.NAME.label}
                value={formData.name || ''}
                onChange={(value) => updateField('name', value)}
                placeholder={STRINGS.NAME.placeholder}
                required
                error={validationErrors.name}
                disabled={!hasUpdatePermissions}
              />
            </SettingsFormSection>
          )}

          {/* Type-specific settings */}
          {selectedType?.toLowerCase() === 'file' && (
            <FileBlobStoreSettings
              data={formData}
              onChange={updateNestedField}
              disabled={!hasUpdatePermissions}
              isEdit={isEdit}
            />
          )}

          {selectedType?.toLowerCase() === 's3' && (
            <S3BlobStoreSettings
              data={formData}
              onChange={updateNestedField}
              disabled={!hasUpdatePermissions}
              isEdit={isEdit}
            />
          )}

          {selectedType?.toLowerCase() === 'azure' && (
            <AzureBlobStoreSettings
              data={formData}
              onChange={updateNestedField}
              disabled={!hasUpdatePermissions}
              isEdit={isEdit}
            />
          )}

          {selectedType?.toLowerCase() === 'google' && (
            <GoogleBlobStoreSettings
              data={formData}
              onChange={updateNestedField}
              disabled={!hasUpdatePermissions}
              isEdit={isEdit}
            />
          )}

          {selectedType?.toLowerCase() === 'group' && (
            <GroupBlobStoreSettings
              data={formData}
              onChange={updateNestedField}
              disabled={!hasUpdatePermissions}
              isEdit={isEdit}
            />
          )}

          {/* Soft Quota - for all types */}
          {selectedType && (
            <SettingsFormSection
              title={STRINGS.SOFT_QUOTA.title}
              description={STRINGS.SOFT_QUOTA.description}
            >
              <SettingsCheckbox
                name="blobstore-softquota-enabled"
                label={STRINGS.SOFT_QUOTA.ENABLED}
                checked={formData.softQuota?.enabled || false}
                onChange={handleSoftQuotaToggle}
                disabled={!hasUpdatePermissions}
              />

              {formData.softQuota?.enabled && (
                <>
                  <SettingsSelect
                    name="blobstore-softquota-type"
                    label={STRINGS.SOFT_QUOTA.TYPE.label}
                    value={formData.softQuota?.type || ''}
                    onChange={handleSoftQuotaTypeChange}
                    options={[
                      { value: '', label: STRINGS.SOFT_QUOTA.TYPE.placeholder },
                      ...quotaTypes.map((q) => ({ value: q.id, label: q.name })),
                    ]}
                    required
                    error={validationErrors['softQuota.type']}
                    disabled={!hasUpdatePermissions}
                  />

                  <SettingsTextInput
                    name="blobstore-softquota-limit"
                    label={STRINGS.SOFT_QUOTA.LIMIT.label}
                    value={formData.softQuota?.limit?.toString() || ''}
                    onChange={handleSoftQuotaLimitChange}
                    placeholder={STRINGS.SOFT_QUOTA.LIMIT.placeholder}
                    type="number"
                    required
                    error={validationErrors['softQuota.limit']}
                    disabled={!hasUpdatePermissions}
                  />
                </>
              )}
            </SettingsFormSection>
          )}
        </SettingsForm>

        {/* Delete Confirmation Modal */}
        {showDeleteConfirm && (
          <div className="blob-stores-page__modal-overlay">
            <div className="blob-stores-page__modal">
              <h3>{STRINGS.CONFIRM_DELETE.title}</h3>
              <p>{STRINGS.CONFIRM_DELETE.message}</p>
              <div className="blob-stores-page__modal-actions">
                <SettingsButton
                  variant="ghost"
                  onClick={() => setShowDeleteConfirm(false)}
                  disabled={deleting}
                  testId="modal-cancel"
                >
                  {STRINGS.CONFIRM_DELETE.cancel}
                </SettingsButton>
                <SettingsButton
                  variant="danger"
                  onClick={handleDelete}
                  disabled={deleting}
                  testId="modal-confirm-delete"
                >
                  {deleting ? BlobStoresStrings.PAGE.DELETING : STRINGS.CONFIRM_DELETE.confirm}
                </SettingsButton>
              </div>
            </div>
          </div>
        )}

        {/* Convert to Group Modal */}
        {showConvertModal && (
          <ConvertToGroupModal
            blobStoreName={routeState.blobStoreName!}
            onConfirm={handleConvertToGroup}
            onCancel={() => setShowConvertModal(false)}
            promoting={promoting}
          />
        )}

        {/* Dangerous Edit Confirmation Dialog */}
        <DangerousEditConfirmDialog
          open={showDangerousEditDialog}
          onClose={() => setShowDangerousEditDialog(false)}
          onConfirm={handleConfirmDangerousEdit}
          blobStoreName={routeState.blobStoreName || ''}
          changedFields={dangerousFieldsChanged}
        />
      </Box>
    );
  };

  // Create mode: use 4-step wizard
  if (routeState.viewMode === 'create') {
    return (
      <Box className="blob-stores-page" data-testid="blob-stores-page" data-view="create">
        <BlobStoreWizardCreate onBack={handleBack} />
      </Box>
    );
  }

  return (
    <Box
      className="blob-stores-page"
      data-testid="blob-stores-page"
      data-view={routeState.viewMode}
    >
      {routeState.viewMode === 'list' && renderListView()}
      {routeState.viewMode === 'detail' && renderFormView()}
    </Box>
  );
}

