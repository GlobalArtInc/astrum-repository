import {
  Button,
  DropdownMenu,
  Heading,
  IconButton,
  TextField as RadixTextField,
  Select,
  Table,
  Text,
} from '@radix-ui/themes';
import { ArrowDown, ArrowUp, ArrowUpDown, Ellipsis, Funnel, Search, X } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router';
import { useT } from '../../i18n';
import {
  CheckList,
  copyText,
  ErrorState,
  FilterLayout,
  FilterSection,
  formatName,
  NARROW,
  ResetFiltersButton,
  useDocumentTitle,
  useMediaQuery,
  useToast,
} from '../../ui';
import { activate } from '../search/SearchResults';
import type { UploadRepository } from './model';
import { useUploadTargets } from './targets';
import styles from './Upload.module.css';

type SortKey = 'name' | 'format';
type Direction = 'asc' | 'desc';
type Sort = { key: SortKey; direction: Direction } | null;

const DEFAULT_SORT: Sort = { key: 'name', direction: 'asc' };
const VISIBLE_FORMATS = 7;

function sortRows(rows: UploadRepository[], sort: Sort): UploadRepository[] {
  if (!sort) {
    return rows;
  }
  const value = (row: UploadRepository) => (sort.key === 'name' ? row.name : formatName(row.format));
  const sorted = [...rows].sort(
    (left, right) => value(left).localeCompare(value(right)) || left.name.localeCompare(right.name),
  );
  return sort.direction === 'asc' ? sorted : sorted.reverse();
}

function SortHeader({
  label,
  column,
  sort,
  onSort,
}: {
  label: string;
  column: SortKey;
  sort: Sort;
  onSort(next: Sort): void;
}) {
  const t = useT();
  const active = sort?.key === column ? sort.direction : null;
  const Icon = active === 'asc' ? ArrowUp : active === 'desc' ? ArrowDown : ArrowUpDown;
  const next: Sort =
    active === null
      ? { key: column, direction: 'asc' }
      : active === 'asc'
        ? { key: column, direction: 'desc' }
        : null;
  const hint =
    active === 'asc'
      ? t('upload.list.sortedAscending', { column: label })
      : active === 'desc'
        ? t('upload.list.sortedDescending', { column: label })
        : t('upload.list.sortBy', { column: label });
  const sortNow = () => onSort(next);
  return (
    <span
      role="button"
      tabIndex={0}
      className={styles.sortHeader}
      title={hint}
      aria-label={hint}
      onClick={sortNow}
      onKeyDown={activate(sortNow)}
    >
      {label}
      <Icon size={14} aria-hidden="true" />
    </span>
  );
}

export default function UploadListPage() {
  const t = useT();
  const navigate = useNavigate();
  const toast = useToast();
  const narrow = useMediaQuery(NARROW);
  useDocumentTitle(t('upload.list.title'));
  const { targets, loading, error, retry } = useUploadTargets();
  const [filter, setFilter] = useState('');
  const [formats, setFormats] = useState<string[]>([]);
  const [sort, setSort] = useState<Sort>(DEFAULT_SORT);
  const [sheet, setSheet] = useState(false);

  const formatOptions = useMemo(() => {
    const counts = new Map<string, number>();
    for (const target of targets) {
      counts.set(target.format, (counts.get(target.format) ?? 0) + 1);
    }
    return [...counts.entries()]
      .map(([value, count]) => ({ value, label: formatName(value), count }))
      .sort((left, right) => left.label.localeCompare(right.label));
  }, [targets]);

  const wanted = filter.trim().toLowerCase();
  const shown = sortRows(
    targets.filter(
      (target) =>
        (formats.length === 0 || formats.includes(target.format)) &&
        (wanted === '' || target.name.toLowerCase().includes(wanted)),
    ),
    sort,
  );

  const reset = () => {
    setFormats([]);
    setSort(DEFAULT_SORT);
  };
  const resetAll = () => {
    reset();
    setFilter('');
  };

  const sortKey = sort?.key ?? 'name';
  const sortDirection = sort?.direction ?? 'asc';
  const filters = (
    <div className={styles.filters}>
      <ResetFiltersButton onClick={reset} />
      <div className={styles.filterSections}>
        <FilterSection title={t('upload.list.sort')} weight="semibold">
          <div className={styles.sortSelects}>
            <Select.Root
              value={sortKey}
              onValueChange={(key) => setSort({ key: key as SortKey, direction: sortDirection })}
            >
              <Select.Trigger aria-label={t('upload.list.sort')} />
              <Select.Content position="popper">
                <Select.Item value="name">{t('upload.list.name')}</Select.Item>
                <Select.Item value="format">{t('upload.list.format')}</Select.Item>
              </Select.Content>
            </Select.Root>
            <Select.Root
              value={sortDirection}
              onValueChange={(direction) => setSort({ key: sortKey, direction: direction as Direction })}
            >
              <Select.Trigger aria-label={t('upload.list.direction')} />
              <Select.Content position="popper">
                <Select.Item value="asc">{t('upload.list.ascending')}</Select.Item>
                <Select.Item value="desc">{t('upload.list.descending')}</Select.Item>
              </Select.Content>
            </Select.Root>
          </div>
        </FilterSection>
        <FilterSection title={t('upload.list.format')}>
          <CheckList
            options={formatOptions}
            selected={formats}
            onChange={setFormats}
            visible={VISIBLE_FORMATS}
            clearable
            className={styles.formatChecks}
          />
        </FilterSection>
      </div>
    </div>
  );

  return (
    <div className={styles.listPage}>
      <FilterLayout
        label={t('upload.list.filtersLabel')}
        contentLabel={t('upload.list.contentLabel')}
        width={250}
        gap={32}
        filters={filters}
        sheetOpen={sheet}
        onSheetOpenChange={setSheet}
        onResetAll={resetAll}
      >
        <div className={styles.listHead}>
          <Heading as="h1" size="6" weight="bold">
            {t('upload.list.title')}
          </Heading>
          {!loading && !error && (
            <Text as="p" size="2" className={styles.gray}>
              {t('upload.list.count', { count: shown.length })}
            </Text>
          )}
        </div>
        <div className={styles.toolbar} role="toolbar" aria-label={t('upload.list.toolbar')}>
          <RadixTextField.Root
            className={styles.filterInput}
            value={filter}
            placeholder={t('upload.list.filterPlaceholder')}
            aria-label={t('upload.list.filter')}
            onChange={(event) => setFilter(event.target.value)}
          >
            <RadixTextField.Slot>
              <Search size={14} aria-hidden="true" />
            </RadixTextField.Slot>
            {filter !== '' && (
              <RadixTextField.Slot side="right">
                <IconButton
                  type="button"
                  size="1"
                  variant="ghost"
                  color="gray"
                  aria-label={t('upload.list.clearFilter')}
                  onClick={() => setFilter('')}
                >
                  <X size={14} aria-hidden="true" />
                </IconButton>
              </RadixTextField.Slot>
            )}
          </RadixTextField.Root>
          {narrow && (
            <div className={styles.phoneTools}>
              <Button
                type="button"
                variant="outline"
                color="gray"
                aria-label={t('upload.list.openFilters')}
                onClick={() => setSheet(true)}
              >
                <Funnel size={14} aria-hidden="true" />
                {t('upload.list.filterButton')}
              </Button>
              <Select.Root
                value={`${sortKey}-${sortDirection}`}
                onValueChange={(value) => {
                  const [key, direction] = value.split('-') as [SortKey, Direction];
                  setSort({ key, direction });
                }}
              >
                <Select.Trigger className={styles.phoneSort} aria-label={t('upload.list.sort')} />
                <Select.Content position="popper">
                  <Select.Item value="name-asc">{t('upload.list.nameAscending')}</Select.Item>
                  <Select.Item value="name-desc">{t('upload.list.nameDescending')}</Select.Item>
                  <Select.Item value="format-asc">{t('upload.list.formatAscending')}</Select.Item>
                  <Select.Item value="format-desc">{t('upload.list.formatDescending')}</Select.Item>
                </Select.Content>
              </Select.Root>
            </div>
          )}
        </div>
        {error ? (
          <ErrorState error={error} onRetry={retry} />
        ) : loading ? (
          <div className={styles.empty} role="status">
            <Text size="3" className={styles.gray}>
              {t('upload.list.loading')}
            </Text>
          </div>
        ) : shown.length === 0 ? (
          <div className={styles.empty}>
            <Text size="3" className={styles.gray}>
              {t('upload.list.none')}
            </Text>
          </div>
        ) : (
          <div className={styles.tableWrap}>
            <Table.Root variant="surface" size="2">
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell>
                    <SortHeader label={t('upload.list.name')} column="name" sort={sort} onSort={setSort} />
                  </Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>
                    <SortHeader
                      label={t('upload.list.format')}
                      column="format"
                      sort={sort}
                      onSort={setSort}
                    />
                  </Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell
                    justify="end"
                    width="80px"
                    aria-label={t('upload.list.rowActions')}
                  />
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {shown.map((target) => {
                  const open = () => navigate(`/upload/${encodeURIComponent(target.name)}`);
                  return (
                    <Table.Row
                      key={target.name}
                      className={styles.row}
                      role="button"
                      tabIndex={0}
                      aria-label={t('upload.list.uploadTo', { name: target.name })}
                      onClick={open}
                      onKeyDown={activate(open)}
                    >
                      <Table.Cell>
                        <Text weight="medium">{target.name}</Text>
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="2">{formatName(target.format)}</Text>
                      </Table.Cell>
                      <Table.Cell justify="end" onClick={(event) => event.stopPropagation()}>
                        <DropdownMenu.Root>
                          <DropdownMenu.Trigger>
                            <IconButton
                              type="button"
                              size="1"
                              variant="ghost"
                              color="gray"
                              aria-label={t('upload.list.actionsFor', { name: target.name })}
                            >
                              <Ellipsis size={16} aria-hidden="true" />
                            </IconButton>
                          </DropdownMenu.Trigger>
                          <DropdownMenu.Content align="end">
                            <DropdownMenu.Item onSelect={open}>
                              {t('upload.list.uploadTo', { name: target.name })}
                            </DropdownMenu.Item>
                            <DropdownMenu.Item
                              onSelect={() => {
                                void copyText(target.url).then(
                                  () => toast.success(t('upload.list.copied')),
                                  () => undefined,
                                );
                              }}
                            >
                              {t('upload.list.copyUrl')}
                            </DropdownMenu.Item>
                          </DropdownMenu.Content>
                        </DropdownMenu.Root>
                      </Table.Cell>
                    </Table.Row>
                  );
                })}
              </Table.Body>
            </Table.Root>
          </div>
        )}
      </FilterLayout>
    </div>
  );
}
