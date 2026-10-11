

import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Box, Flex, Text, Checkbox } from '@radix-ui/themes';
import { Trash2, AlertCircle, Info, Loader2 } from 'lucide-react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsTextArea,
  SettingsSelect,
  SettingsButton,
  SettingsAlert,
  SettingsTransferList,
} from '../../../../shared/form';
import { CleanupPolicyPreview } from './CleanupPolicyPreview';
import { CleanupPolicyDryRun } from './CleanupPolicyDryRun';
import { useCleanupPoliciesApi } from './useCleanupPoliciesApi';
import { useCleanupPolicyForm } from './useCleanupPolicyForm';
import {
  CleanupPolicy,
  CleanupPolicyFormData,
  FormatCriteria,
  RepositoryOption,
  RELEASE_TYPES,
  NOTES_MAX_LENGTH,
  isRetainSupportedFormat,
  isRepositoriesFieldSupportedFormat,
} from './types';

import './CleanupPolicyForm.scss';
import CleanupPoliciesStrings from '../../../../constants/pages/settings/repository/CleanupPoliciesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface CleanupPolicyFormProps {
  policy?: CleanupPolicy;
  isCreate: boolean;
  formatCriteria: FormatCriteria[];
  canDelete?: boolean;
  onSave: (data: CleanupPolicyFormData) => Promise<void>;
  onDelete?: () => void;
  onCancel: () => void;
  loading?: boolean;
  error?: string;
}

/**
 * CleanupPolicyForm - Create/Edit form for cleanup policies
 * Now uses XState for state management via useCleanupPolicyForm hook
 */
export function CleanupPolicyForm({
  policy,
  isCreate,
  formatCriteria,
  canDelete,
  onSave,
  onDelete,
  onCancel,
  loading,
  error,
}: CleanupPolicyFormProps) {
  const {
    createCleanupPolicy,
    updateCleanupPolicy,
    isPreviewEnabled,
    isRetainEnabled: checkRetainEnabled,
    isRetainAllFormatsEnabled,
    fetchRepositories,
    isRenormalizationInProgress,
  } = useCleanupPoliciesApi();

  const isEnhancedCleanupEnabled = useMemo(() => isRetainAllFormatsEnabled(), [isRetainAllFormatsEnabled]);

  const [availableRepos, setAvailableRepos] = useState<RepositoryOption[]>([]);
  const [selectedRepos, setSelectedRepos] = useState<string[]>([]);
  const [reposLoading, setReposLoading] = useState(false);

  // Baseline repo selection used to compute dirty state for the Repositories section.
  // The XState form machine doesn't track repositories, so we OR this into pristine ourselves.
  // This MUST be state (not a ref) so that `reposDirty` recomputes after we refresh the
  // baseline post-save — ref mutations don't trigger renders and would leave reposDirty stale.
  const [initialRepos, setInitialRepos] = useState<string[]>([]);

  const reposDirty = useMemo(() => {
    const a = [...selectedRepos].sort();
    const b = [...initialRepos].sort();
    if (a.length !== b.length) return true;
    return a.some((v, i) => v !== b[i]);
  }, [selectedRepos, initialRepos]);

  // Refs let the save-time getter passed to the form hook read the latest
  // selection without the hook needing `selectedRepos`/`reposDirty` in deps
  // (which would break the memoized machine).
  const selectedReposRef = useRef<string[]>(selectedRepos);
  const reposDirtyRef = useRef<boolean>(reposDirty);
  useEffect(() => {
    selectedReposRef.current = selectedRepos;
  }, [selectedRepos]);
  useEffect(() => {
    reposDirtyRef.current = reposDirty;
  }, [reposDirty]);

  // Use XState form hook. We pass a getter for `repositories` so the
  // machine's save service can pull the latest selection at save time —
  // the machine itself does not track repository state. The selection is
  // included in the backend payload per the contract:
  //   - returns undefined -> omit field (preserve existing attachments)
  //   - returns []        -> clear all attachments
  //   - returns [a, b]    -> set attachments to exactly {a, b}
  const {
    form,
    criteriaEnabled,
    changeFormat,
    toggleCriteria,
    changeReleaseType,
  } = useCleanupPolicyForm({
    policyName: isCreate ? undefined : policy?.name,
    policy: policy || null,
    formatCriteria,
    onSave: async () => {
      // Post-save hook: refresh baseline so subsequent edits compute dirty
      // state against the just-persisted selection.
      setInitialRepos((current) => [...selectedReposRef.current ?? current]);
    },
    onCancel,
    createPolicy: createCleanupPolicy,
    updatePolicy: updateCleanupPolicy,
    getRepositories: () => {
      const fmt = (form?.data as CleanupPolicyFormData | undefined)?.format;
      if (!fmt) return undefined;
      if (!isRetainAllFormatsEnabled()) return undefined;
      if (!isRepositoriesFieldSupportedFormat(fmt)) return undefined;
      const sel = selectedReposRef.current;
      const dirty = reposDirtyRef.current;
      return sel.length > 0 || dirty ? sel : undefined;
    },
  });

  const formData = form.data as CleanupPolicyFormData;

  // Load repos when format changes
  useEffect(() => {
    if (formData.format) {
      setReposLoading(true);
      setSelectedRepos([]);
      setInitialRepos([]);
      fetchRepositories(formData.format)
        .then(setAvailableRepos)
        .catch(() => setAvailableRepos([]))
        .finally(() => setReposLoading(false));
    } else {
      setAvailableRepos([]);
      setSelectedRepos([]);
      setInitialRepos([]);
    }
  }, [formData.format, fetchRepositories]);

  // Pre-populate repos on edit. The backend now includes `repositories` inline
  // on GET /cleanup-policies/{name}, so we read directly from the loaded policy
  // instead of issuing a second request. We deliberately key only on
  // `policy?.name` and `formData.format` so that in-session edits to
  // `selectedRepos` are not clobbered when a parent re-renders with a
  // referentially-new `policy.repositories` array carrying the same values.
  useEffect(() => {
    if (isCreate || !policy?.name || !formData.format) {
      return;
    }
    const incomingRepos = policy.repositories ?? [];
    setSelectedRepos(incomingRepos);
    setInitialRepos(incomingRepos);
  }, [isCreate, policy?.name, formData.format, policy?.repositories]);

  const handleReposChange = useCallback(
    (items: RepositoryOption[]) => {
      setSelectedRepos(items.map((r) => r.name));
    },
    []
  );

  // Get current format's available criteria
  const currentFormatCriteria = useMemo(() => {
    return formatCriteria.find((fc) => fc.id === formData.format);
  }, [formatCriteria, formData.format]);

  const isFieldApplicable = useCallback(
    (fieldId: string): boolean => {
      return currentFormatCriteria?.availableCriteria?.includes(fieldId) ?? false;
    },
    [currentFormatCriteria]
  );

  const showRetain = useMemo(() => {
    return isRetainSupportedFormat(formData.format) &&
      (checkRetainEnabled(formData.format) || isRenormalizationInProgress(formData.format));
  }, [formData.format, checkRetainEnabled, isRenormalizationInProgress]);

  const hasOtherCriteriaSelected = useMemo(() => {
    return !!(
      formData.criteriaLastBlobUpdated ||
      formData.criteriaLastDownloaded ||
      formData.criteriaAssetRegex
    );
  }, [formData.criteriaLastBlobUpdated, formData.criteriaLastDownloaded, formData.criteriaAssetRegex]);

  const isMaven = formData.format === 'maven2';

  // Whether the current format exposes the Release Type dropdown. Formats like docker do not,
  // so the release-type requirement must not gate Retain for them.
  const releaseTypeApplicable = isFieldApplicable('isPrerelease');

  // For maven2, only the "Releases" release type enables Retain — Pre-Releases/Snapshots and the
  // combined default are not valid because Maven treats only -SNAPSHOT as a pre-release, and the
  // retain-by-version semantics are meaningful only against release versions.
  // For other formats that show the Release Type dropdown, either "Releases" or "Pre-Releases"
  // satisfies the requirement.
  // For formats without a Release Type dropdown (e.g. docker), this requirement is treated as
  // satisfied so Retain depends only on a cleanup criterion being selected.
  const hasReleaseTypeSelected = useMemo(() => {
    if (!releaseTypeApplicable) {
      return true;
    }
    if (isMaven) {
      return formData.criteriaReleaseType === 'RELEASES';
    }
    return !!formData.criteriaReleaseType && formData.criteriaReleaseType !== '';
  }, [formData.criteriaReleaseType, isMaven, releaseTypeApplicable]);

  const isRetainDisabled = useMemo(() => {
    return !hasOtherCriteriaSelected || !hasReleaseTypeSelected;
  }, [hasOtherCriteriaSelected, hasReleaseTypeSelected]);

  const retainUnavailableMessage = useMemo(() => {
    if (!formData.format || !isRetainSupportedFormat(formData.format)) {
      return null;
    }
    if (!isRetainAllFormatsEnabled() || !hasReleaseTypeSelected) {
      return null;
    }
    if (isRenormalizationInProgress(formData.format)) {
      return CleanupPoliciesStrings.FORM.ABILITY_RETAIN_SELECT_NUMBER_MOST;
    }
    return null;
  }, [formData.format, isRetainAllFormatsEnabled, hasReleaseTypeSelected, isRenormalizationInProgress]);

  const retainDisabledReason = useMemo(() => {
    if (isMaven && !hasReleaseTypeSelected && !hasOtherCriteriaSelected) {
      return CleanupPoliciesStrings.FORM.OPTION_ONLY_APPLICABLE_RELEASES_SELECT;
    }
    if (isMaven && !hasReleaseTypeSelected) {
      return CleanupPoliciesStrings.FORM.OPTION_ONLY_APPLICABLE_RELEASES_SELECT_2;
    }
    if (!hasOtherCriteriaSelected && !hasReleaseTypeSelected) {
      return CleanupPoliciesStrings.FORM.SELECT_RELEASE_TYPE_LEAST_ONE;
    }
    if (!hasReleaseTypeSelected) {
      return CleanupPoliciesStrings.FORM.SELECT_RELEASE_TYPE_RELEASES_PRE;
    }
    if (!hasOtherCriteriaSelected) {
      return CleanupPoliciesStrings.FORM.ONLY_AFTER_SELECTING_CLEANUP_CRITERIA;
    }
    return null;
  }, [hasOtherCriteriaSelected, hasReleaseTypeSelected, isMaven]);

  const showPreview = useMemo(() => isPreviewEnabled(), [isPreviewEnabled]);

  const showRepositoriesSection = useMemo(
    () => isEnhancedCleanupEnabled && !!formData.format && isRepositoriesFieldSupportedFormat(formData.format),
    [isEnhancedCleanupEnabled, formData.format]
  );

  // Check if criteria section should show
  const showCriteriaSection = useMemo(() => {
    return (
      formData.format &&
      (isFieldApplicable('lastBlobUpdated') ||
        isFieldApplicable('lastDownloaded') ||
        isFieldApplicable('regex'))
    );
  }, [formData.format, isFieldApplicable]);

  // Show loading state
  if (form.isLoading || loading) {
    return (
      <Box className="cleanup-policy-form">
        <Flex align="center" justify="center" gap="2" p="4">
          <Loader2 size={24} className="cleanup-policy-form__spinner" />
          <Text>{CommonStrings.MESSAGES.LOADING}</Text>
        </Flex>
      </Box>
    );
  }

  return (
    <Box className="cleanup-policy-form">
      <SettingsForm
          onSubmit={() => form.send('SUBMIT')}
          onCancel={onCancel}
          loading={form.isSaving}
          pristine={form.isPristine && !reposDirty}
          error={error || form.saveError || undefined}
          submitLabel={isCreate ? CommonStrings.ACTIONS.CREATE : CommonStrings.ACTIONS.SAVE}
          cancelLabel={CommonStrings.ACTIONS.CANCEL}
          testId="cleanup-policy-form"
          submitAnalyticsId={isCreate ? 'as-cleanup-policy-create' : 'as-cleanup-policy-save'}
          footerExtra={
            canDelete && onDelete ? (
              <SettingsButton
                variant="danger"
                onClick={onDelete}
                icon={Trash2}
                disabled={form.isSaving}
                testId="form-delete"
                data-analytics-id="as-cleanup-policy-delete"
              >
                {CommonStrings.ACTIONS.DELETE}
              </SettingsButton>
            ) : undefined
          }
        >
          <Box mb="4">
            <Text as="p" size="2">
              {CleanupPoliciesStrings.FORM.INTRO}
            </Text>
          </Box>

          {/* Basic Info Section */}
          <SettingsFormSection title={CleanupPoliciesStrings.FORM.POLICY_SETTINGS}>
            <SettingsTextInput
              {...form.field('name')}
              label={CommonStrings.LABELS.NAME}
              sublabel={CleanupPoliciesStrings.FORM.USE_UNIQUE_NAME_CLEANUP_POLICY}
              helpText={CleanupPoliciesStrings.FORM.UNIQUE_POLICY_NAME_E_G}
              disabled={!isCreate}
              required
              placeholder="delete-old-snapshots"
            />

            <SettingsSelect
              name="format"
              label={CommonStrings.LABELS.FORMAT}
              helpText={CleanupPoliciesStrings.FORM.FORMAT_CLEANUP_POLICY_CAN_APPLIED}
              value={formData.format || ''}
              onChange={changeFormat}
              required
              placeholder={CleanupPoliciesStrings.FORM.SELECT_FORMAT_PLACEHOLDER}
              options={formatCriteria.map((fc) => ({
                value: fc.id,
                label: fc.name,
              }))}
            />

            <SettingsTextArea
              {...form.field('notes')}
              label={CommonStrings.LABELS.DESCRIPTION}
              helpText={CleanupPoliciesStrings.FORM.OPTIONAL_NOTES_ABOUT_WHY_POLICY}
              maxLength={NOTES_MAX_LENGTH}
              className="cleanup-policy-form__notes-textarea"
            />
          </SettingsFormSection>

          {/* Repository Selection — gated by feature flag and supported format set */}
          {showRepositoriesSection && (
            <SettingsFormSection title={CleanupPoliciesStrings.FORM.REPOSITORIES} description={CleanupPoliciesStrings.FORM.SELECT_REPOSITORIES_POLICY_APPLY}>
              {reposLoading ? (
                <Flex align="center" gap="2">
                  <Loader2 size={16} className="cleanup-policy-form__spinner" />
                  <Text size="2">{CleanupPoliciesStrings.FORM.LOADING_REPOSITORIES}</Text>
                </Flex>
              ) : availableRepos.length === 0 ? (
                <Text size="2" color="gray">{CleanupPoliciesStrings.FORM.NO_REPOSITORIES_AVAILABLE_FORMAT}</Text>
              ) : (
                <SettingsTransferList
                  name="cleanup-policy-repositories"
                  label={CleanupPoliciesStrings.FORM.REPOSITORIES}
                  availableItems={availableRepos}
                  selectedItems={availableRepos.filter((r) => selectedRepos.includes(r.name))}
                  onChange={handleReposChange}
                  availableLabel={CleanupPoliciesStrings.FORM.AVAILABLE_REPOSITORIES}
                  selectedLabel={CleanupPoliciesStrings.FORM.APPLIED_REPOSITORIES}
                  getItemId={(r: RepositoryOption) => r.name}
                  getItemLabel={(r: RepositoryOption) => r.name}
                  helpText={CleanupPoliciesStrings.FORM.SELECT_REPOSITORIES_APPLY_POLICY}
                  testId="cleanup-policy-repositories-transfer-list"
                />
              )}
            </SettingsFormSection>
          )}

          {/* Release Type (for applicable formats) */}
          {isFieldApplicable('isPrerelease') && (
            <SettingsFormSection title={CleanupPoliciesStrings.FORM.RELEASE_TYPE}>
              <SettingsSelect
                name="releaseType"
                label={CleanupPoliciesStrings.FORM.RELEASE_TYPE}
                helpText={CleanupPoliciesStrings.FORM.REMOVE_COMPONENTS_FOLLOWING_RELEASE_TYPE}
                value={formData.criteriaReleaseType || ''}
                onChange={changeReleaseType}
                placeholder=""
                options={Object.values(RELEASE_TYPES).map((type) => ({
                  value: type.id,
                  label: type.label,
                }))}
              />
            </SettingsFormSection>
          )}

          {/* Cleanup Criteria Section */}
          {showCriteriaSection && (
            <SettingsFormSection
              title={CleanupPoliciesStrings.FORM.CLEANUP_CRITERIA_TITLE}
              description={CleanupPoliciesStrings.FORM.REMOVE_ALL_COMPONENTS_MATCH_ALL}
            >
              {form.touched?.criteriaSelected && form.validationErrors?.criteriaSelected && (
                <Box className="cleanup-policy-form__criteria-error">
                  <SettingsAlert type="error">
                    <Flex align="center" gap="2">
                      <AlertCircle size={16} />
                      {form.validationErrors.criteriaSelected}
                    </Flex>
                  </SettingsAlert>
                </Box>
              )}

              {/* Component Age */}
              {isFieldApplicable('lastBlobUpdated') && (
                <Box className="cleanup-policy-form__criteria-item">
                  <Flex align="start" gap="3">
                    <Checkbox
                      data-testid="checkbox-criteria-lastBlobUpdated"
                      checked={criteriaEnabled?.lastBlobUpdated || false}
                      onCheckedChange={(checked) =>
                        toggleCriteria('lastBlobUpdated', checked === true)
                      }
                    />
                    <Box className="cleanup-policy-form__criteria-content">
                      <Text as="label" size="2" weight="medium">
                        {CleanupPoliciesStrings.FORM.COMPONENT_AGE}
                      </Text>
                      <Text as="p" size="1" color="gray">
                        {CleanupPoliciesStrings.FORM.COMPONENTS_PUBLISHED_OVER_X_DAYS}
                      </Text>
                      {criteriaEnabled?.lastBlobUpdated && (
                        <Box className="cleanup-policy-form__criteria-input">
                          <SettingsTextInput
                            {...form.field('criteriaLastBlobUpdated')}
                            label=""
                            placeholder="e.g 100 days"
                            type="number"
                          />
                        </Box>
                      )}
                    </Box>
                  </Flex>
                </Box>
              )}

              {/* Component Usage */}
              {isFieldApplicable('lastDownloaded') && (
                <Box className="cleanup-policy-form__criteria-item">
                  <Flex align="start" gap="3">
                    <Checkbox
                      data-testid="checkbox-criteria-lastDownloaded"
                      checked={criteriaEnabled?.lastDownloaded || false}
                      onCheckedChange={(checked) =>
                        toggleCriteria('lastDownloaded', checked === true)
                      }
                    />
                    <Box className="cleanup-policy-form__criteria-content">
                      <Text as="label" size="2" weight="medium">
                        {CleanupPoliciesStrings.FORM.COMPONENT_USAGE}
                      </Text>
                      <Text as="p" size="1" color="gray">
                        {CleanupPoliciesStrings.FORM.COMPONENTS_DOWNLOADED_X_AMOUNT_DAYS}
                      </Text>
                      {criteriaEnabled?.lastDownloaded && (
                        <Box className="cleanup-policy-form__criteria-input">
                          <SettingsTextInput
                            {...form.field('criteriaLastDownloaded')}
                            label=""
                            placeholder="e.g 100 days"
                            type="number"
                          />
                        </Box>
                      )}
                    </Box>
                  </Flex>
                </Box>
              )}

              {/* Asset Name Matcher */}
              {isFieldApplicable('regex') && (
                <Box className="cleanup-policy-form__criteria-item">
                  <Flex align="start" gap="3">
                    <Checkbox
                      data-testid="checkbox-criteria-assetRegex"
                      checked={criteriaEnabled?.assetRegex || false}
                      onCheckedChange={(checked) =>
                        toggleCriteria('assetRegex', checked === true)
                      }
                    />
                    <Box className="cleanup-policy-form__criteria-content">
                      <Text as="label" size="2" weight="medium">
                        {CleanupPoliciesStrings.FORM.ASSET_NAME_MATCHER}
                      </Text>
                      <Text as="p" size="1" color="gray">
                        {CleanupPoliciesStrings.FORM.REMOVE_COMPONENTS_HAVE_LEAST_ONE}
                      </Text>
                      {criteriaEnabled?.assetRegex && (
                        <Box className="cleanup-policy-form__criteria-input cleanup-policy-form__criteria-input--wide">
                          <SettingsTextInput
                            {...form.field('criteriaAssetRegex')}
                            label=""
                            placeholder="e.g. .*-SNAPSHOT.*"
                          />
                        </Box>
                      )}
                    </Box>
                  </Flex>
                </Box>
              )}

              {/* Retain Versions (Exclusion Criteria) */}
              {retainUnavailableMessage && (
                <Box mt="3">
                  <SettingsAlert type="info">
                    {retainUnavailableMessage}
                  </SettingsAlert>
                </Box>
              )}
              {showRetain && (
                <Box className="cleanup-policy-form__criteria-item cleanup-policy-form__exclusion">
                  <Flex align="center" gap="2" className="cleanup-policy-form__exclusion-header">
                    <Info size={16} />
                    <Text size="2" weight="medium">
                      {CleanupPoliciesStrings.FORM.EXCEPT_DO_NOT_REMOVE_ANY}
                    </Text>
                  </Flex>
                  {retainDisabledReason && (
                    <Box mt="3">
                      <SettingsAlert type="info">
                        {retainDisabledReason}
                      </SettingsAlert>
                    </Box>
                  )}
                  <Flex align="start" gap="3" mt="3">
                    <Checkbox
                      data-testid="checkbox-criteria-retain"
                      checked={criteriaEnabled?.retain || false}
                      onCheckedChange={(checked) =>
                        toggleCriteria('retain', checked === true)
                      }
                      disabled={isRetainDisabled}
                    />
                    <Box className="cleanup-policy-form__criteria-content" data-disabled={isRetainDisabled || undefined}>
                      <Text as="label" size="2" weight="medium">
                        {CleanupPoliciesStrings.FORM.NUMBER_VERSIONS}
                      </Text>
                      <Text as="p" size="1" color="gray">
                        {formData.format === 'docker'
                          ? CleanupPoliciesStrings.FORM.KEEP_LATEST_BY_AGE
                          : CleanupPoliciesStrings.FORM.KEEP_LATEST_BY_VERSION}
                      </Text>
                      {criteriaEnabled?.retain && !isRetainDisabled && (
                        <Box className="cleanup-policy-form__criteria-input">
                          <SettingsTextInput
                            {...form.field('retain')}
                            label=""
                            placeholder="e.g. 5"
                            type="number"
                          />
                        </Box>
                      )}
                    </Box>
                  </Flex>
                </Box>
              )}
            </SettingsFormSection>
          )}

          {/* Preview Section */}
          {showPreview && formData.format && (
            <SettingsFormSection title={CleanupPoliciesStrings.FORM.PREVIEW_CLEANUP_POLICY_RESULTS}>
              <CleanupPolicyDryRun
                policyData={formData}
                policyName={policy?.name}
                selectedRepositories={selectedRepos}
              />
            </SettingsFormSection>
          )}
        </SettingsForm>

      {/* Legacy Preview (for non-PostgreSQL) */}
      {!showPreview && formData.format && (
        <CleanupPolicyPreview policyData={formData} selectedRepositories={selectedRepos} />
      )}
    </Box>
  );
}

export default CleanupPolicyForm;
