

import React, { useCallback, useState, useMemo } from 'react';
import { useRouter, useCurrentStateAndParams } from '@uirouter/react';
import { AlertCircle, Trash2, ArrowLeft, AlertTriangle, RefreshCw, } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
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
import { useBlobStore, useBlobStorePromote } from './useBlobStores';
import { useBlobStoreForm } from './useBlobStoreForm';
import FileBlobStoreSettings from './FileBlobStoreSettings';
import S3BlobStoreSettings from './S3BlobStoreSettings';
import AzureBlobStoreSettings from './AzureBlobStoreSettings';
import GoogleBlobStoreSettings from './GoogleBlobStoreSettings';
import GroupBlobStoreSettings from './GroupBlobStoreSettings';
import ConvertToGroupModal from './ConvertToGroupModal';
import { DeleteConfirmationModal } from '../../../../shared/modals/DeleteConfirmationModal';
import { DangerousEditConfirmDialog } from './DangerousEditConfirmDialog';
import { hasDangerousFieldChanges, getDangerousFieldsChanged } from './dangerousFields';
import type { BlobStoreFormData, SoftQuota } from './types';
import './BlobStoresForm.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const STRINGS = {
  CREATE_TITLE: BlobStoresStrings.FORM.CREATE_BLOB_STORE,
  EDIT_TITLE: (name: string) => BlobStoresStrings.FORM.EDIT(name),
  EDIT_DESCRIPTION: (type: string) => BlobStoresStrings.FORM.BLOB_STORE(type),
  EDIT_WARNING: BlobStoresStrings.FORM.UPDATING_BLOB_STORE_CONFIGURATION_CAUSE,
  CONVERT_TO_GROUP_BUTTON: BlobStoresStrings.FORM.CONVERT_GROUP,
  TYPE: {
    label: CommonStrings.LABELS.TYPE,
    sublabel: BlobStoresStrings.FORM.SELECT_TYPE_BLOB_STORE
  },
  NAME: {
    label: CommonStrings.LABELS.NAME,
    placeholder: BlobStoresStrings.FORM.ENTER_BLOB_STORE_NAME
  },
  SOFT_QUOTA: {
    title: BlobStoresStrings.FORM.SOFT_QUOTA,
    description: BlobStoresStrings.FORM.RAISES_ALERT_WHEN_BLOB_STORE,
    ENABLED: BlobStoresStrings.FORM.ENABLE_SOFT_QUOTA,
    TYPE: {
      label: BlobStoresStrings.FORM.CONSTRAINT_TYPE,
      placeholder: BlobStoresStrings.FORM.SELECT_CONSTRAINT_TYPE
    },
    LIMIT: {
      label: BlobStoresStrings.FORM.CONSTRAINT_LIMIT_MB,
      placeholder: BlobStoresStrings.FORM.ENTER_LIMIT_MB
    }
  },
  DELETE: {
    button: CommonStrings.ACTIONS.DELETE,
    tooltip: (repositoryUsage: number, blobStoreUsage: number, hasPermission: boolean) => {
      if (!hasPermission) {
        return BlobStoresStrings.FORM.YOU_DONT_HAVE_PERMISSION_DELETE;
      }
      return BlobStoresStrings.FORM.BLOB_STORE_USE_REPOSITORIES_BLOB(repositoryUsage, blobStoreUsage);
    }
  },
};

const SPACE_USED_QUOTA_ID = 'spaceUsedQuota';

/**
 * BlobStoresForm - Create/Edit form for blob stores
 * Now uses XState for state management via useBlobStoreForm hook
 */
export default function BlobStoresForm() {
  const router = useRouter();
  const { params } = useCurrentStateAndParams();
  const name = params.name ? decodeURIComponent(params.name) : undefined;
  const typeFromUrl = params.type ? decodeURIComponent(params.type) : undefined;
  const isEdit = Boolean(name);

  // Permissions
  const hasUpdatePermissions = ExtJS.checkPermission(Permissions.BLOB_STORES.UPDATE);
  const hasDeletePermission = ExtJS.checkPermission(Permissions.BLOB_STORES.DELETE);

  // API functions from existing hook
  const { save, remove } = useBlobStore(name, typeFromUrl);
  const { promoting, promote } = useBlobStorePromote();

  // Navigation
  const handleBack = useCallback(() => {
    router.stateService.go('preview.admin.repository.blobstores.list');
  }, [router]);

  // Use XState form hook
  const {
    form,
    blobStoreTypes: types,
    quotaTypes,
    usage,
  } = useBlobStoreForm({
    blobStoreName: name,
    blobStoreType: typeFromUrl,
    saveBlobStore: save,
    updateBlobStore: save,
    deleteBlobStore: remove ? async (_blobName: string) => { await remove(); } : undefined,
    onCancel: handleBack,
  });

  const formData = form.data as BlobStoreFormData;
  const selectedType = formData.type || typeFromUrl || '';
  const { blobStoreUsage = 0, repositoryUsage = 0 } = usage || {};

  // Convert to group state
  const [showConvertModal, setShowConvertModal] = useState(false);

  // Dangerous edit confirmation state
  const [showDangerousEditDialog, setShowDangerousEditDialog] = useState(false);

  // Update field via machine
  const updateField = useCallback((field: string, value: unknown) => {
    form.send({ type: 'UPDATE', name: field, value });
  }, [form]);

  // Handle name field changes - also updates path for file blob stores
  const handleNameChange = useCallback((value: string) => {
    form.send({ type: 'UPDATE', name: 'name', value });
    // Also send NAME_CHANGE to trigger path update for file blob stores
    form.send({ type: 'NAME_CHANGE', value });
  }, [form]);

  const updateNestedField = useCallback((path: string, value: unknown) => {
    // For nested fields like 'bucketConfiguration.bucket.name', we need to construct the nested object
    const parts = path.split('.');
    if (parts.length === 1) {
      form.send({ type: 'UPDATE', name: path, value });
      return;
    }
    // Build the nested update
    const currentData = formData as any;
    const topKey = parts[0];
    const current = currentData[topKey] || {};
    let obj = { ...current };
    let ref = obj;
    for (let i = 1; i < parts.length - 1; i++) {
      ref[parts[i]] = { ...(ref[parts[i]] || {}) };
      ref = ref[parts[i]];
    }
    ref[parts[parts.length - 1]] = value;
    form.send({ type: 'UPDATE', name: topKey, value: obj });
  }, [form, formData]);

  // Soft quota handling
  const handleSoftQuotaToggle = useCallback((enabled: boolean) => {
    const currentQuota = (formData as any).softQuota || { enabled: false };
    const newQuota: SoftQuota = {
      ...currentQuota,
      enabled,
      type: enabled ? (currentQuota.type || SPACE_USED_QUOTA_ID) : undefined,
      limit: enabled ? currentQuota.limit : undefined
    };
    updateField('softQuota', newQuota);
  }, [formData, updateField]);

  const handleSoftQuotaTypeChange = useCallback((type: string) => {
    updateField('softQuota', {
      ...(formData as any).softQuota,
      type
    });
  }, [formData, updateField]);

  const handleSoftQuotaLimitChange = useCallback((limit: string) => {
    updateField('softQuota', {
      ...(formData as any).softQuota,
      limit: limit ? parseInt(limit, 10) : undefined
    });
  }, [formData, updateField]);

  // Handle type change via machine
  const handleTypeChange = useCallback((typeId: string) => {
    form.send({ type: 'TYPE_CHANGE', value: typeId } as any);
  }, [form]);

  // Convert to group
  const handleConvertToGroup = useCallback(async (newGroupName: string) => {
    if (!name) return;
    try {
      await promote(name, newGroupName);
      setShowConvertModal(false);
      handleBack();
    } catch (_err) {
      // Error handled by promote
    }
  }, [name, promote, handleBack]);

  // Dangerous edit detection - intercept save for edit mode
  const pristineData = (form.state as any).context.pristineData as BlobStoreFormData | undefined;

  const handleSave = useCallback(() => {
    if (isEdit && pristineData && hasDangerousFieldChanges(pristineData, formData, selectedType)) {
      setShowDangerousEditDialog(true);
    } else {
      form.send('SUBMIT');
    }
  }, [isEdit, pristineData, formData, selectedType, form]);

  const handleConfirmDangerousEdit = useCallback(() => {
    setShowDangerousEditDialog(false);
    form.send('SUBMIT');
  }, [form]);

  const dangerousFieldsChanged = useMemo(() => {
    if (!isEdit || !pristineData) return [];
    return getDangerousFieldsChanged(pristineData, formData, selectedType);
  }, [isEdit, pristineData, formData, selectedType]);

  // Get current type configuration
  const currentType = types.find(t => t.id === selectedType || t.id === typeFromUrl);
  const canDelete = blobStoreUsage === 0 && repositoryUsage === 0 && hasDeletePermission;
  const canConvertToGroup = isEdit && currentType?.id !== 'group' && types.some(t => t.id === 'group');

  if (form.isLoading) {
    return (
      <div className="blob-stores-form">
        <div className="blob-stores-form__loading">
          <div className="blob-stores-form__spinner" />
          <span>{CommonStrings.MESSAGES.LOADING}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="blob-stores-form">
      <SettingsForm
        testId="blobstore-form"
        title={isEdit ? STRINGS.EDIT_TITLE(name!) : STRINGS.CREATE_TITLE}
        description={isEdit && currentType ? STRINGS.EDIT_DESCRIPTION(currentType.name) : undefined}
        onSave={hasUpdatePermissions ? handleSave : undefined}
        onCancel={handleBack}
        saving={form.isSaving}
        dirty={!form.isPristine}
        headerActions={
          <>
            <SettingsButton
              variant="ghost"
              onClick={handleBack}
              icon={ArrowLeft}
            >
              {CommonStrings.ACTIONS.BACK}
            </SettingsButton>
            {canConvertToGroup && hasUpdatePermissions && (
              <SettingsButton
                variant="secondary"
                onClick={() => setShowConvertModal(true)}
                disabled={promoting}
                icon={RefreshCw}
              >
                {STRINGS.CONVERT_TO_GROUP_BUTTON}
              </SettingsButton>
            )}
          </>
        }
        footerExtra={
          isEdit ? (
            <SettingsButton
              variant="danger"
              onClick={() => form.send('CONFIRM_DELETE')}
              disabled={!canDelete || form.isDeleting}
              icon={Trash2}
              title={!canDelete ? STRINGS.DELETE.tooltip(repositoryUsage, blobStoreUsage, hasDeletePermission) : undefined}
            >
              {STRINGS.DELETE.button}
            </SettingsButton>
          ) : undefined
        }
      >
        {form.saveError && (
          <SettingsAlert variant="error" icon={<AlertCircle size={16} />}>
            {form.saveError}
          </SettingsAlert>
        )}

        {isEdit && hasUpdatePermissions && (
          <SettingsAlert variant="warning" icon={<AlertTriangle size={16} />}>
            {STRINGS.EDIT_WARNING}
          </SettingsAlert>
        )}

        {/* Type Selection - only for create */}
        {!isEdit && (
          <SettingsFormSection title={BlobStoresStrings.FORM.BLOB_STORE_TYPE}>
            {types.length === 0 ? (
              <SettingsAlert variant="error" icon={<AlertCircle size={16} />}>
                {BlobStoresStrings.FORM.NO_BLOB_STORE_TYPES_AVAILABLE}
              </SettingsAlert>
            ) : (
              <SettingsSelect
                label={STRINGS.TYPE.label}
                helpText={STRINGS.TYPE.sublabel}
                value={selectedType}
                onChange={handleTypeChange}
                options={[
                  { value: '', label: BlobStoresStrings.FORM.SELECT_TYPE },
                  ...types.map(t => ({ value: t.id, label: t.name }))
                ]}
                required
                error={form.touched?.type ? form.validationErrors?.type : undefined}
                disabled={!hasUpdatePermissions}
              />
            )}
          </SettingsFormSection>
        )}

        {/* Name - only for create */}
        {!isEdit && selectedType && (
          <SettingsFormSection title={BlobStoresStrings.FORM.BASIC_CONFIGURATION}>
            <SettingsTextInput
              {...form.field('name')}
              label={STRINGS.NAME.label}
              placeholder={STRINGS.NAME.placeholder}
              helpText={BlobStoresStrings.FORM.UNIQUE_NAME_BLOB_STORE_CANNOT}
              required
              disabled={!hasUpdatePermissions}
              onChange={handleNameChange}
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
            errors={form.validationErrors}
          />
        )}

        {selectedType?.toLowerCase() === 's3' && (
          <S3BlobStoreSettings
            data={formData}
            onChange={updateNestedField}
            disabled={!hasUpdatePermissions}
            isEdit={isEdit}
            errors={form.validationErrors}
          />
        )}

        {selectedType?.toLowerCase() === 'azure' && (
          <AzureBlobStoreSettings
            data={formData}
            onChange={updateNestedField}
            disabled={!hasUpdatePermissions}
            isEdit={isEdit}
            errors={form.validationErrors}
          />
        )}

        {selectedType?.toLowerCase() === 'google' && (
          <GoogleBlobStoreSettings
            data={formData}
            onChange={updateNestedField}
            disabled={!hasUpdatePermissions}
            isEdit={isEdit}
            errors={form.validationErrors}
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
              label={STRINGS.SOFT_QUOTA.ENABLED}
              checked={(formData as any).softQuota?.enabled}
              onChange={handleSoftQuotaToggle}
              disabled={!hasUpdatePermissions}
            />

            {(formData as any).softQuota?.enabled && (
              <>
                <SettingsSelect
                  label={STRINGS.SOFT_QUOTA.TYPE.label}
                  value={(formData as any).softQuota?.type || ''}
                  onChange={handleSoftQuotaTypeChange}
                  options={[
                    { value: '', label: STRINGS.SOFT_QUOTA.TYPE.placeholder },
                    ...quotaTypes.map(q => ({ value: q.id, label: q.name }))
                  ]}
                  required
                  disabled={!hasUpdatePermissions}
                />

                <SettingsTextInput
                  label={STRINGS.SOFT_QUOTA.LIMIT.label}
                  value={(formData as any).softQuota?.limit?.toString() || ''}
                  onChange={handleSoftQuotaLimitChange}
                  placeholder={STRINGS.SOFT_QUOTA.LIMIT.placeholder}
                  type="number"
                  required
                  disabled={!hasUpdatePermissions}
                />
              </>
            )}
          </SettingsFormSection>
        )}
      </SettingsForm>

      {/* Convert to Group Modal */}
      {showConvertModal && (
        <ConvertToGroupModal
          blobStoreName={name!}
          onConfirm={handleConvertToGroup}
          onCancel={() => setShowConvertModal(false)}
          promoting={promoting}
        />
      )}

      {/* Delete Confirmation Modal */}
      <DeleteConfirmationModal
        open={(form.state as any).context.showDeleteModal}
        onClose={() => form.send('HIDE_DELETE_MODAL')}
        onConfirm={() => form.send('DELETE')}
        entityName={name}
        entityType="blob store"
        loading={form.isDeleting}
      />

      {/* Dangerous Edit Confirmation Dialog */}
      <DangerousEditConfirmDialog
        open={showDangerousEditDialog}
        onClose={() => setShowDangerousEditDialog(false)}
        onConfirm={handleConfirmDangerousEdit}
        blobStoreName={name || ''}
        changedFields={dangerousFieldsChanged}
      />
    </div>
  );
}
