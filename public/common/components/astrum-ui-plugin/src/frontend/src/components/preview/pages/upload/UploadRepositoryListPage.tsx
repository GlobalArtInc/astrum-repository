

import React, { useCallback, useState } from 'react';
import {
  Box,
  Button,
  Flex,
  Grid,
  IconButton,
  Select,
  TextField,
} from '@radix-ui/themes';
import { Filter, Search, X } from 'lucide-react';
import { useRouter } from '@uirouter/react';

import { PageHeader } from '../../shared';
import { useUploadableRepositories } from './hooks/useUploadableRepositories';
import { UploadFilterSidebar } from './components/UploadFilterSidebar';
import { UploadRepositoryTable } from './components/UploadRepositoryTable';
import { MobileFilterDrawer } from '../../shared/MobileFilterDrawer/MobileFilterDrawer';
import { UPLOAD_STRINGS } from './upload.types';
import type { SortColumn } from './upload.types';

import '../search/unified/SearchSidebar.scss';
import UploadStrings from '../../constants/pages/upload/UploadStrings';
import CommonStrings from '../../constants/CommonStrings';
import plural from '../../../../interface/plural';

const UPLOAD_FORM_ROUTE = 'preview.browse.upload.form';

/**
 * UploadRepositoryListPage displays a list of hosted repositories that support file uploads.
 * Layout matches Browse list: Grid with filter sidebar, header, and table.
 */
export function UploadRepositoryListPage(): JSX.Element {
  const router = useRouter();
  const [showMobileFilters, setShowMobileFilters] = useState(false);

  const {
    repositories,
    loading,
    error,
    filterText,
    selectedFormats,
    formatOptions,
    sortColumn,
    sortDirection,
    handleSortChange,
    handleFilterChange,
    clearFilter,
    setFormats,
    clearAllFilters,
  } = useUploadableRepositories();

  const handleSelectRepository = useCallback(
    (repositoryName: string) => {
      router.stateService.go(UPLOAD_FORM_ROUTE, { repoName: repositoryName });
    },
    [router]
  );

  const handleInputChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      handleFilterChange(event.target.value);
    },
    [handleFilterChange]
  );

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Escape') {
        clearFilter();
      }
    },
    [clearFilter]
  );

  const handleFormatsChange = useCallback(
    (values: string[]) => {
      setFormats(values);
    },
    [setFormats]
  );

  const sortField = sortColumn ?? 'name';
  const sortDir = sortDirection ?? 'asc';

  const SORT_VALUES = [
    { value: 'name-asc', label: UploadStrings.LIST_PAGE.NAME_ASCENDING },
    { value: 'name-desc', label: UploadStrings.LIST_PAGE.NAME_DESCENDING },
    { value: 'format-asc', label: UploadStrings.LIST_PAGE.FORMAT_ASCENDING },
    { value: 'format-desc', label: UploadStrings.LIST_PAGE.FORMAT_DESCENDING },
  ] as const;

  const sortValue = `${sortField}-${sortDir}`;

  const renderSortDropdown = () => (
    <Select.Root
      value={sortValue}
      onValueChange={(v) => {
        const [col, dir] = v.split('-') as [SortColumn, 'asc' | 'desc'];
        handleSortChange(col, dir);
      }}
      size="2"
    >
      <Select.Trigger
        placeholder={UploadStrings.LIST_PAGE.NAME_ASCENDING_PLACEHOLDER}
        className="upload-list-page__sort-trigger"
      />
      <Select.Content position="popper" side="bottom" avoidCollisions={false} sideOffset={4}>
        {SORT_VALUES.map((opt) => (
          <Select.Item key={opt.value} value={opt.value}>
            {opt.label}
          </Select.Item>
        ))}
      </Select.Content>
    </Select.Root>
  );

  const filterBarContent = (
    <UploadFilterSidebar
      formatOptions={formatOptions}
      selectedFormats={selectedFormats}
      sortColumn={sortColumn}
      sortDirection={sortDirection}
      onFormatsChange={handleFormatsChange}
      onSortChange={handleSortChange}
      onResetFilters={clearAllFilters}
      disabled={loading}
    />
  );

  return (
    <Box
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
      width="100%"
      className="upload-list-page"
      data-testid="upload-page"
    >
      <Flex direction="column" gap="6" width="100%" className="upload-list-page__outer">
        <Grid
          columns={{ initial: '1', sm: '250px 1fr' }}
          gap="6"
          width="100%"
          className="upload-list-page__grid"
        >
          {/* Filter Sidebar — hidden on mobile */}
          <Box
            className="filter-bar upload-list-page__sidebar"
            display={{ initial: 'none', sm: 'block' }}
            role="complementary"
            aria-label={UploadStrings.LIST_PAGE.FILTER_BAR_ARIA}
          >
            <aside className="search-sidebar">{filterBarContent}</aside>
          </Box>

          {/* Main Content */}
          <Box
            className="page-content"
            minWidth="0"
            width="100%"
            role="main"
            aria-label={UploadStrings.LIST_PAGE.PAGE_CONTENT_ARIA}
          >
            <Box mb="4">
              <PageHeader
                title={UploadStrings.LIST_PAGE.UPLOAD}
                description={plural(UploadStrings.LIST_PAGE.REPOSITORY_COUNT, repositories.length)}
              />
            </Box>
            <Box mb="4" role="toolbar" aria-label={UploadStrings.LIST_PAGE.ACTIONS_BAR_ARIA}>
              <Flex
                className="actions-bar upload-list-page__actions-bar"
                align="center"
                gap="3"
                wrap="wrap"
              >
                <Box className="upload-list-page__filter-input-wrapper">
                  <TextField.Root
                    placeholder={UPLOAD_STRINGS.filterPlaceholder}
                    value={filterText}
                    onChange={handleInputChange}
                    onKeyDown={handleKeyDown}
                    size="2"
                    className="upload-list-page__filter-input"
                    aria-label={UploadStrings.LIST_PAGE.FILTER_REPOSITORIES_NAME_ARIA}
                  >
                    <TextField.Slot>
                      <Search size={14} />
                    </TextField.Slot>
                    {filterText && (
                      <TextField.Slot side="right">
                        <IconButton
                          variant="ghost"
                          color="gray"
                          size="1"
                          onClick={clearFilter}
                          aria-label={UploadStrings.LIST_PAGE.CLEAR_FILTER_ARIA}
                        >
                          <X size={14} />
                        </IconButton>
                      </TextField.Slot>
                    )}
                  </TextField.Root>
                </Box>
                {/* The sidebar holds the filters and the sort from the width it shows at */}
                <Flex
                  gap="3"
                  align="center"
                  display={{ initial: 'flex', sm: 'none' }}
                  data-testid="upload-list-mobile-controls"
                >
                  <Button
                    variant="outline"
                    size="2"
                    color="gray"
                    onClick={() => setShowMobileFilters(true)}
                    aria-label={UploadStrings.LIST_PAGE.OPEN_FILTERS_ARIA}
                  >
                    <Filter size={14} />
                    {CommonStrings.ACTIONS.FILTER}
                  </Button>
                  {renderSortDropdown()}
                </Flex>
              </Flex>
            </Box>

            {/* Table */}
            <UploadRepositoryTable
              repositories={repositories}
              loading={loading}
              error={error}
              sortColumn={sortField}
              sortDirection={sortDir}
              onSort={(col, dir) => handleSortChange(col, dir)}
              onSelect={handleSelectRepository}
            />
          </Box>
        </Grid>
      </Flex>

      <MobileFilterDrawer
        isOpen={showMobileFilters}
        onClose={() => setShowMobileFilters(false)}
        title={CommonStrings.ACTIONS.FILTER}
        onClearAll={clearAllFilters}
      >
        {filterBarContent}
      </MobileFilterDrawer>
    </Box>
  );
}

export default UploadRepositoryListPage;
