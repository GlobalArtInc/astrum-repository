

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { Box, Flex, Text, Table, TextField, Card, Callout } from '@radix-ui/themes';
import { Search, ChevronUp, ChevronDown, AlertTriangle } from 'lucide-react';

import { SettingsSelect, SettingsButton, SettingsAlert } from '../../../../shared/form';
import { useCleanupPoliciesApi } from './useCleanupPoliciesApi';
import { CleanupPolicySummary } from './CleanupPolicySummary';
import {
  CleanupPolicyFormData,
  RepositoryOption,
  PreviewComponent,
  isRepositoriesFieldSupportedFormat,
} from './types';

import './CleanupPolicyPreview.scss';
import CleanupPoliciesStrings from '../../../../constants/pages/settings/repository/CleanupPoliciesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface CleanupPolicyPreviewProps {
  policyData: CleanupPolicyFormData;
  selectedRepositories?: string[];
}

type SortField = 'name' | 'group' | 'version';
type SortDirection = 'asc' | 'desc';

/**
 * CleanupPolicyPreview - Preview components that would be deleted by a cleanup policy
 */
export function CleanupPolicyPreview({ policyData, selectedRepositories = [] }: CleanupPolicyPreviewProps) {
  const [repositories, setRepositories] = useState<RepositoryOption[]>([]);
  const [selectedRepository, setSelectedRepository] = useState('');
  const [previewResults, setPreviewResults] = useState<PreviewComponent[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [filter, setFilter] = useState('');
  const [sortField, setSortField] = useState<SortField>('name');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [isLoadingRepos, setIsLoadingRepos] = useState(false);
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);
  const [repoError, setRepoError] = useState<string | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [showWarning, setShowWarning] = useState(false);

  const { fetchRepositories, previewCleanupPolicy } = useCleanupPoliciesApi();

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
      setPreviewResults([]);
      setTotalCount(0);
      setRepoError(null);
      setIsLoadingRepos(false);
    } else if (isRepositoriesFieldSupportedFormat(policyData.format)) {
      // Format supports per-repository application but none are applied yet —
      // show an empty dropdown instead of listing every repo.
      setRepositories([]);
      setSelectedRepository('');
      setPreviewResults([]);
      setTotalCount(0);
      setRepoError(null);
      setIsLoadingRepos(false);
    } else {
      setIsLoadingRepos(true);
      setRepoError(null);
      setSelectedRepository('');
      setPreviewResults([]);
      setTotalCount(0);

      fetchRepositories(policyData.format)
        .then(setRepositories)
        .catch((err) => setRepoError(err.message))
        .finally(() => setIsLoadingRepos(false));
    }
  }, [policyData.format, fetchRepositories, _selectedReposKey]);

  // Check if preview is available
  const isPreviewAvailable = useMemo(() => {
    return (
      Boolean(selectedRepository) &&
      (Boolean(policyData.criteriaLastBlobUpdated) ||
        Boolean(policyData.criteriaLastDownloaded) ||
        Boolean(policyData.criteriaReleaseType) ||
        Boolean(policyData.criteriaAssetRegex))
    );
  }, [
    selectedRepository,
    policyData.criteriaLastBlobUpdated,
    policyData.criteriaLastDownloaded,
    policyData.criteriaReleaseType,
    policyData.criteriaAssetRegex,
  ]);

  // Handle preview
  const handlePreview = useCallback(async () => {
    if (!isPreviewAvailable) return;

    // Validate regex pattern before sending request
    if (policyData.criteriaAssetRegex) {
      try {
        new RegExp(policyData.criteriaAssetRegex);
      } catch {
        setPreviewError(CleanupPoliciesStrings.PREVIEW.INVALID_REGULAR_EXPRESSION_PATTERN_USE);
        return;
      }
    }

    setIsLoadingPreview(true);
    setPreviewError(null);

    try {
      const result = await previewCleanupPolicy(selectedRepository, policyData);
      setPreviewResults(result.components);
      setTotalCount(result.total);
      setShowWarning(result.components.length > 0);
    } catch (err: any) {
      setPreviewError(err.message);
    } finally {
      setIsLoadingPreview(false);
    }
  }, [isPreviewAvailable, selectedRepository, policyData, previewCleanupPolicy]);

  // Filter and sort results
  const filteredResults = useMemo(() => {
    // Deduplicate by (name, group, version) — backend may emit duplicate rows when
    // the retain (Number of Versions) exception is combined with the asset name
    // matcher because the underlying SQL joins to the asset table.
    const seen = new Set<string>();
    let result = previewResults.filter((item) => {
      const key = `${item.name}\u0000${item.group ?? ''}\u0000${item.version ?? ''}`;
      if (seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    });

    // Apply filter
    if (filter) {
      const lowerFilter = filter.toLowerCase();
      result = result.filter(
        (item) =>
          item.name.toLowerCase().includes(lowerFilter) ||
          (item.group?.toLowerCase().includes(lowerFilter)) ||
          (item.version?.toLowerCase().includes(lowerFilter))
      );
    }

    // Apply sort
    if (sortField && sortDirection) {
      result = [...result].sort((a, b) => {
        const aVal = (a[sortField] || '').toLowerCase();
        const bVal = (b[sortField] || '').toLowerCase();
        const cmp = aVal.localeCompare(bVal);
        return sortDirection === 'asc' ? cmp : -cmp;
      });
    }

    return result;
  }, [previewResults, filter, sortField, sortDirection]);

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

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) return null;
    return sortDirection === 'asc' ? (
      <ChevronUp size={14} className="sort-icon" />
    ) : (
      <ChevronDown size={14} className="sort-icon" />
    );
  };

  return (
    <Card className="cleanup-policy-preview">
      <Text as="h3" size="4" weight="medium" className="cleanup-policy-preview__title">
        {CleanupPoliciesStrings.PREVIEW.CLEANUP_POLICY_PREVIEW}
      </Text>

      {/* Repository Selection */}
      <Box className="cleanup-policy-preview__controls">
        <SettingsSelect
          name="previewRepository"
          label={CleanupPoliciesStrings.PREVIEW.PREVIEW_REPOSITORY}
          helpText={CleanupPoliciesStrings.PREVIEW.SELECT_REPOSITORY_PREVIEW_WHAT_MIGHT}
          value={selectedRepository}
          onChange={setSelectedRepository}
          disabled={isLoadingRepos || !policyData.format}
          placeholder={CleanupPoliciesStrings.PREVIEW.SELECT_REPOSITORY_PLACEHOLDER}
          options={repositories.map((repo) => ({
            value: repo.id,
            label: repo.name,
          }))}
        />

        <SettingsButton
          variant="secondary"
          onClick={handlePreview}
          disabled={!isPreviewAvailable || isLoadingPreview}
          className="cleanup-policy-preview__button"
        >
          {isLoadingPreview ? CommonStrings.MESSAGES.LOADING : CleanupPoliciesStrings.PREVIEW.PREVIEW}
        </SettingsButton>
      </Box>

      <CleanupPolicySummary policyData={policyData} repository={selectedRepository} />

      {repoError && (
        <SettingsAlert type="error" className="cleanup-policy-preview__error">
          {repoError}
        </SettingsAlert>
      )}

      {previewError && (
        <SettingsAlert type="error" className="cleanup-policy-preview__error">
          {previewError}
        </SettingsAlert>
      )}

      {/* Warning Message */}
      {showWarning && (
        <Callout.Root color="amber" className="cleanup-policy-preview__warning">
          <Callout.Icon>
            <AlertTriangle size={16} />
          </Callout.Icon>
          <Callout.Text>
            <Text size="2">
              {CleanupPoliciesStrings.PREVIEW.RESULTS_MAY_ONLY_SAMPLE_WHAT}
            </Text>
            <Text size="2" className="cleanup-policy-preview__count">
              {totalCount === 0
                ? CleanupPoliciesStrings.PREVIEW.CRITERIA_MATCHED_NO_COMPONENTS
                : totalCount > 0
                ? CleanupPoliciesStrings.PREVIEW.COMPONENT_COUNT_MATCHING_CRITERIA_VIEWING(filteredResults.length, totalCount)
                : CleanupPoliciesStrings.PREVIEW.COMPONENT_COUNT_MATCHING_CRITERIA_VIEWING_2(filteredResults.length)}
            </Text>
          </Callout.Text>
        </Callout.Root>
      )}

      {/* Search/Filter */}
      <Box className="cleanup-policy-preview__toolbar">
        <TextField.Root
          placeholder={CleanupPoliciesStrings.PREVIEW.FILTER_RESULTS_PLACEHOLDER}
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="cleanup-policy-preview__search"
        >
          <TextField.Slot>
            <Search size={16} />
          </TextField.Slot>
        </TextField.Root>
      </Box>

      {/* Results Table */}
      <Table.Root className="cleanup-policy-preview__table">
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell
              className="cleanup-policy-preview__sortable-header"
              onClick={() => handleSort('name')}
            >
              <Flex align="center" gap="1">
                {CommonStrings.LABELS.NAME} {getSortIcon('name')}
              </Flex>
            </Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell
              className="cleanup-policy-preview__sortable-header"
              onClick={() => handleSort('group')}
            >
              <Flex align="center" gap="1">
                {CleanupPoliciesStrings.PREVIEW.GROUP} {getSortIcon('group')}
              </Flex>
            </Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell
              className="cleanup-policy-preview__sortable-header"
              onClick={() => handleSort('version')}
            >
              <Flex align="center" gap="1">
                {CleanupPoliciesStrings.PREVIEW.VERSION} {getSortIcon('version')}
              </Flex>
            </Table.ColumnHeaderCell>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {isLoadingPreview ? (
            <Table.Row>
              <Table.Cell colSpan={3}>
                <Flex align="center" justify="center" py="4">
                  <Text size="2" color="gray">
                    {CleanupPoliciesStrings.PREVIEW.LOADING_PREVIEW}
                  </Text>
                </Flex>
              </Table.Cell>
            </Table.Row>
          ) : filteredResults.length === 0 ? (
            <Table.Row>
              <Table.Cell colSpan={3}>
                <Flex align="center" justify="center" py="4">
                  <Text size="2" color="gray">
                    {previewResults.length === 0
                      ? CleanupPoliciesStrings.PREVIEW.NO_ASSETS_REPOSITORY_MATCHED_CRITERIA
                      : CleanupPoliciesStrings.PREVIEW.NO_RESULTS_MATCH_FILTER}
                  </Text>
                </Flex>
              </Table.Cell>
            </Table.Row>
          ) : (
            filteredResults.map((item, index) => (
              <Table.Row key={`${item.name}-${item.group}-${item.version}-${index}`}>
                <Table.Cell>{item.name}</Table.Cell>
                <Table.Cell>{item.group || '—'}</Table.Cell>
                <Table.Cell>{item.version || '—'}</Table.Cell>
              </Table.Row>
            ))
          )}
        </Table.Body>
      </Table.Root>
    </Card>
  );
}

export default CleanupPolicyPreview;

