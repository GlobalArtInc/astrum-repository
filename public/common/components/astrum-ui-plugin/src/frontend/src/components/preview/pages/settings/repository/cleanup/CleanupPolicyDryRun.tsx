

import React, { useState, useEffect, useMemo, } from 'react';
import { Box, Flex, Text, Tooltip, Table, Callout } from '@radix-ui/themes';
import { Download, AlertTriangle, Loader2 } from 'lucide-react';

import { SettingsSelect, SettingsButton, SettingsAlert } from '../../../../shared/form';
import UIStrings from '../../../../../../constants/UIStrings';
import { useCleanupPoliciesApi } from './useCleanupPoliciesApi';
import { CleanupPolicySummary } from './CleanupPolicySummary';
import {
  CleanupPolicyFormData,
  RepositoryOption,
  PreviewComponent,
  isRepositoriesFieldSupportedFormat,
} from './types';

import './CleanupPolicyDryRun.scss';
import CleanupPoliciesStrings from '../../../../constants/pages/settings/repository/CleanupPoliciesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const { DRY_RUN } = UIStrings.CLEANUP_POLICIES;

interface CleanupPolicyDryRunProps {
  policyData: CleanupPolicyFormData;
  policyName?: string;
  selectedRepositories?: string[];
}

/**
 * CleanupPolicyDryRun - Generate CSV report of components that would be deleted
 */
export function CleanupPolicyDryRun({ policyData, policyName, selectedRepositories = [] }: CleanupPolicyDryRunProps) {
  const [repositories, setRepositories] = useState<RepositoryOption[]>([]);
  const [selectedRepository, setSelectedRepository] = useState('');
  const [isLoadingRepos, setIsLoadingRepos] = useState(false);
  const [repoError, setRepoError] = useState<string | null>(null);
  const [previewResults, setPreviewResults] = useState<PreviewComponent[]>([]);
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  const { fetchRepositories, getDryRunCsvUrl, previewCleanupPolicy } = useCleanupPoliciesApi();

  // Serialize selected repos for stable dependency comparison
  const _selectedReposKey = selectedRepositories.join(',');

  // Load repositories — restrict the dropdown to the Applied Repositories selected
  // in the dual-list selector above. For formats that don't support attaching the
  // policy to specific repositories, fall back to fetching all repos for the format.
  useEffect(() => {
    if (!policyData.format) return;

    if (selectedRepositories.length > 0) {
      setRepositories(selectedRepositories.map((name) => ({ id: name, name })));
      setSelectedRepository('');
      setRepoError(null);
      setIsLoadingRepos(false);
    } else if (isRepositoriesFieldSupportedFormat(policyData.format)) {
      // Format supports per-repository application but none are applied yet —
      // show an empty dropdown instead of listing every repo.
      setRepositories([]);
      setSelectedRepository('');
      setRepoError(null);
      setIsLoadingRepos(false);
    } else {
      setIsLoadingRepos(true);
      setRepoError(null);
      setSelectedRepository('');

      fetchRepositories(policyData.format)
        .then(setRepositories)
        .catch((err) => setRepoError(err.message))
        .finally(() => setIsLoadingRepos(false));
    }
  }, [policyData.format, fetchRepositories, _selectedReposKey]);

  // Check if at least one criteria is selected
  const hasCriteria = useMemo(() => {
    return (
      Boolean(policyData.criteriaLastBlobUpdated) ||
      Boolean(policyData.criteriaLastDownloaded) ||
      Boolean(policyData.criteriaAssetRegex)
    );
  }, [
    policyData.criteriaLastBlobUpdated,
    policyData.criteriaLastDownloaded,
    policyData.criteriaAssetRegex,
  ]);

  // Check if download is available
  const isDownloadAvailable = useMemo(() => {
    return Boolean(selectedRepository) && hasCriteria;
  }, [selectedRepository, hasCriteria]);

  // Generate download URL
  const downloadUrl = useMemo(() => {
    if (!isDownloadAvailable) return '';
    return getDryRunCsvUrl(selectedRepository, policyData, policyName);
  }, [isDownloadAvailable, selectedRepository, policyData, policyName, getDryRunCsvUrl]);

  const tooltipMessage = useMemo(() => {
    if (!(selectedRepository || hasCriteria)) {
      return CleanupPoliciesStrings.DRY_RUN.PLEASE_SELECT_REPOSITORY_LEAST_ONE;
    }
    if (!selectedRepository) {
      return CleanupPoliciesStrings.DRY_RUN.PLEASE_SELECT_REPOSITORY;
    }
    if (!hasCriteria) {
      return CleanupPoliciesStrings.DRY_RUN.PLEASE_SELECT_LEAST_ONE_CLEANUP;
    }
    return '';
  }, [selectedRepository, hasCriteria]);

  // Auto-load preview when repository is selected and criteria exist
  // Use specific policyData fields as dependencies (not the object reference)
  // to ensure re-fetch when individual criteria values change
  useEffect(() => {
    if (selectedRepository && hasCriteria) {
      setIsLoadingPreview(true);
      setPreviewError(null);
      previewCleanupPolicy(selectedRepository, policyData, policyName)
        .then((result) => {
          setPreviewResults(result.components);
        })
        .catch((err) => {
          setPreviewError(err.message);
          setPreviewResults([]);
        })
        .finally(() => setIsLoadingPreview(false));
    } else {
      setPreviewResults([]);
    }
  }, [
    selectedRepository,
    hasCriteria,
    policyData.criteriaLastBlobUpdated,
    policyData.criteriaLastDownloaded,
    policyData.criteriaAssetRegex,
    policyData.criteriaReleaseType,
    policyData.retain,
    policyData.sortBy,
    policyName,
    previewCleanupPolicy,
  ]);

  return (
    <Box className="cleanup-policy-dry-run">
      <Text as="p" size="2" className="cleanup-policy-dry-run__description">
        {CleanupPoliciesStrings.DRY_RUN.EXPORT_SPREADSHEET_LISTING_WHICH_COMPONENTS}
      </Text>

      {repoError && (
        <SettingsAlert type="error" className="cleanup-policy-dry-run__error">
          {repoError}
        </SettingsAlert>
      )}

      <Flex gap="3" align="end" className="cleanup-policy-dry-run__controls">
        <Box className="cleanup-policy-dry-run__select">
          <SettingsSelect
            name="dryRunRepository"
            label=""
            value={selectedRepository}
            onChange={setSelectedRepository}
            disabled={isLoadingRepos || !policyData.format}
            placeholder={CleanupPoliciesStrings.DRY_RUN.SELECT_REPOSITORY_PLACEHOLDER}
            options={repositories.map((repo) => ({
              value: repo.id,
              label: repo.name,
            }))}
          />
        </Box>

        {isDownloadAvailable ? (
          <a
            href={downloadUrl}
            download
            className="cleanup-policy-dry-run__download-link"
          >
            <SettingsButton variant="secondary" icon={Download} data-analytics-id="as-cleanup-policy-preview">
              {CleanupPoliciesStrings.DRY_RUN.GENERATE_CSV_REPORT}
            </SettingsButton>
          </a>
        ) : (
          <Tooltip content={tooltipMessage}>
            <Box>
              <SettingsButton variant="secondary" disabled icon={Download} data-analytics-id="as-cleanup-policy-preview">
                {CleanupPoliciesStrings.DRY_RUN.GENERATE_CSV_REPORT}
              </SettingsButton>
            </Box>
          </Tooltip>
        )}
      </Flex>

      <CleanupPolicySummary
        policyData={policyData}
        repository={selectedRepository}
        policyName={policyName}
      />

      {/* Preview Table — shows components that would be deleted */}
      {selectedRepository && hasCriteria && (
        <Box mt="4">
          {previewError && (
            <SettingsAlert type="error">
              {previewError}
            </SettingsAlert>
          )}

          {!isLoadingPreview && previewResults.length > 0 && (
            <Callout.Root color="amber" mb="3">
              <Callout.Icon>
                <AlertTriangle size={16} />
              </Callout.Icon>
              <Callout.Text>
                <Text size="2">
                  {DRY_RUN.SAMPLE_COUNT(previewResults.length)}
                </Text>
              </Callout.Text>
            </Callout.Root>
          )}

          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>{CommonStrings.LABELS.NAME}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{CleanupPoliciesStrings.DRY_RUN.GROUP}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{CleanupPoliciesStrings.DRY_RUN.VERSION}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {isLoadingPreview ? (
                <Table.Row>
                  <Table.Cell colSpan={3}>
                    <Flex align="center" justify="center" py="4" gap="2">
                      <Loader2 size={16} className="cleanup-policy-dry-run__spinner" />
                      <Text size="2" color="gray">{CleanupPoliciesStrings.DRY_RUN.LOADING_PREVIEW}</Text>
                    </Flex>
                  </Table.Cell>
                </Table.Row>
              ) : previewResults.length === 0 ? (
                <Table.Row>
                  <Table.Cell colSpan={3}>
                    <Flex align="center" justify="center" py="4">
                      <Text size="2" color="gray">
                        {CleanupPoliciesStrings.DRY_RUN.NO_COMPONENTS_MATCH_SELECTED_CRITERIA}
                      </Text>
                    </Flex>
                  </Table.Cell>
                </Table.Row>
              ) : (
                previewResults.map((item, index) => (
                  <Table.Row key={`${item.name}-${item.version}-${index}`}>
                    <Table.Cell>{item.name}</Table.Cell>
                    <Table.Cell>{item.group || '—'}</Table.Cell>
                    <Table.Cell>{item.version || '—'}</Table.Cell>
                  </Table.Row>
                ))
              )}
            </Table.Body>
          </Table.Root>
        </Box>
      )}
    </Box>
  );
}

export default CleanupPolicyDryRun;

