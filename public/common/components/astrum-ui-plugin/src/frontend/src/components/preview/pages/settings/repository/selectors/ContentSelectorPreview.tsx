

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { Box, Flex, Text, Table, TextField } from '@radix-ui/themes';
import { Search } from 'lucide-react';

import { SettingsSelect, SettingsButton, SettingsAlert } from '../../../../shared/form';
import { useContentSelectorsApi } from './useContentSelectorsApi';
import { RepositoryOption } from './types';

import './ContentSelectorPreview.scss';
import ContentSelectorsStrings from '../../../../constants/pages/settings/repository/ContentSelectorsStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface ContentSelectorPreviewProps {
  type: string;
  expression: string;
}

/**
 * ContentSelectorPreview - Preview content that matches a content selector expression
 */
export function ContentSelectorPreview({ type, expression }: ContentSelectorPreviewProps) {
  const [repositories, setRepositories] = useState<RepositoryOption[]>([]);
  const [selectedRepository, setSelectedRepository] = useState('');
  const [previewResults, setPreviewResults] = useState<string[]>([]);
  const [filter, setFilter] = useState('');
  const [isLoadingRepos, setIsLoadingRepos] = useState(false);
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);
  const [repoError, setRepoError] = useState<string | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);

  const { fetchRepositories, previewContentSelector } = useContentSelectorsApi();

  // Load repositories on mount
  useEffect(() => {
    setIsLoadingRepos(true);
    setRepoError(null);

    fetchRepositories()
      .then((repos) => {
        setRepositories(repos);
        // Select "All Repositories" option by default if available
        if (repos.length > 0) {
          const allReposOption = repos.find((r) => r.id === '*');
          if (allReposOption) {
            setSelectedRepository('*');
          }
        }
      })
      .catch((err) => setRepoError(err.message))
      .finally(() => setIsLoadingRepos(false));
  }, [fetchRepositories]);

  // Check if preview is available
  const isPreviewAvailable = useMemo(() => {
    return Boolean(expression.trim());
  }, [expression]);

  // Handle preview
  const handlePreview = useCallback(async () => {
    if (!isPreviewAvailable) return;

    setIsLoadingPreview(true);
    setPreviewError(null);

    try {
      const results = await previewContentSelector(selectedRepository || '*', type, expression);
      setPreviewResults(results);
    } catch (err: any) {
      setPreviewError(err.message);
      setPreviewResults([]);
    } finally {
      setIsLoadingPreview(false);
    }
  }, [isPreviewAvailable, selectedRepository, type, expression, previewContentSelector]);

  // Filter results
  const filteredResults = useMemo(() => {
    if (!filter) return previewResults;
    const lowerFilter = filter.toLowerCase();
    return previewResults.filter((name) => name.toLowerCase().includes(lowerFilter));
  }, [previewResults, filter]);

  return (
    <Box className="content-selector-preview">
      <Text as="p" size="2" className="content-selector-preview__description">
        {ContentSelectorsStrings.PREVIEW.SELECT_REPOSITORY_EVALUATE_CONTENT_SELECTOR}
      </Text>

      {repoError && (
        <SettingsAlert type="error" className="content-selector-preview__error">
          {repoError}
        </SettingsAlert>
      )}

      {previewError && (
        <SettingsAlert type="error" className="content-selector-preview__error">
          {previewError}
        </SettingsAlert>
      )}

      <Flex gap="3" align="end" className="content-selector-preview__controls">
        <Box className="content-selector-preview__select">
          <SettingsSelect
            name="previewRepository"
            label={ContentSelectorsStrings.PREVIEW.PREVIEW_REPOSITORY}
            value={selectedRepository}
            onChange={setSelectedRepository}
            disabled={isLoadingRepos}
            placeholder={ContentSelectorsStrings.PREVIEW.SELECT_OPTION_PLACEHOLDER}
            options={repositories.map((repo) => ({
              value: repo.id,
              label: repo.name,
            }))}
          />
        </Box>

        <SettingsButton
          variant="secondary"
          onClick={handlePreview}
          disabled={!isPreviewAvailable || isLoadingPreview}
          data-analytics-id="as-content-selector-preview"
        >
          {isLoadingPreview ? CommonStrings.MESSAGES.LOADING : ContentSelectorsStrings.PREVIEW.PREVIEW}
        </SettingsButton>
      </Flex>

      {/* Search/Filter */}
      <Box className="content-selector-preview__toolbar">
        <TextField.Root
          placeholder={ContentSelectorsStrings.PREVIEW.FILTER_RESULTS_PLACEHOLDER}
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="content-selector-preview__search"
        >
          <TextField.Slot>
            <Search size={16} />
          </TextField.Slot>
        </TextField.Root>
      </Box>

      {/* Results Table */}
      <Table.Root className="content-selector-preview__table">
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell>{CommonStrings.LABELS.NAME}</Table.ColumnHeaderCell>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {isLoadingPreview ? (
            <Table.Row>
              <Table.Cell>
                <Flex align="center" justify="center" py="4">
                  <Text size="2" color="gray">
                    {ContentSelectorsStrings.PREVIEW.LOADING_PREVIEW}
                  </Text>
                </Flex>
              </Table.Cell>
            </Table.Row>
          ) : filteredResults.length === 0 ? (
            <Table.Row>
              <Table.Cell>
                <Flex align="center" justify="center" py="4">
                  <Text size="2" color="gray">
                    {previewResults.length === 0
                      ? ContentSelectorsStrings.PREVIEW.NO_CONTENT_REPOSITORIES_MATCHED_EXPRESSION
                      : ContentSelectorsStrings.PREVIEW.NO_RESULTS_MATCH_FILTER}
                  </Text>
                </Flex>
              </Table.Cell>
            </Table.Row>
          ) : (
            filteredResults.map((name, index) => (
              <Table.Row key={`${name}-${index}`}>
                <Table.Cell>{name}</Table.Cell>
              </Table.Row>
            ))
          )}
        </Table.Body>
      </Table.Root>
    </Box>
  );
}

export default ContentSelectorPreview;

