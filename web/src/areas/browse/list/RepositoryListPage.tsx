import {
  Badge,
  Button,
  Checkbox,
  Dialog,
  Flex,
  IconButton,
  SegmentedControl,
  Select,
  Skeleton,
  Table,
  Text,
  TextField,
  Tooltip,
} from '@radix-ui/themes';
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  FolderOpen,
  LayoutGrid,
  ListFilter,
  RotateCcw,
  Rows3,
  Search,
  Settings2,
  X,
} from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router';
import { useApi } from '../../../api';
import { formatNumber, useI18n, useT } from '../../../i18n';
import {
  COMPACT,
  EmptyState,
  ErrorState,
  FileSize,
  FormatBadge,
  formatName,
  Page,
  PHONE,
  RelativeTime,
  readStored,
  TypeBadge,
  useMediaQuery,
  writeStored,
} from '../../../ui';
import { OnlineStatus, UrlLine } from '../parts';
import { browseUrl } from '../tree/model';
import { REPOSITORIES_PATH, type RepositoryItem } from '../types';
import {
  activeFilterCount,
  applyFilters,
  DEFAULT_FILTERS,
  type Direction,
  facetCounts,
  type ListFilters,
  readFilters,
  type SortKey,
  toggle,
  writeFilters,
} from './filters';
import styles from './RepositoryList.module.css';

export const VIEW_KEY = 'astrum.browse.view';
type View = 'cards' | 'table';

const SORT_LABELS: Record<SortKey, string> = {
  name: 'browse.list.sort.name',
  format: 'browse.list.sort.format',
  type: 'browse.list.sort.type',
  status: 'browse.list.sort.status',
  components: 'browse.list.sort.components',
  size: 'browse.list.sort.size',
  uploaded: 'browse.list.sort.uploaded',
};

const TYPE_LABELS = {
  hosted: 'browse.list.type.hosted',
  proxy: 'browse.list.type.proxy',
  group: 'browse.list.type.group',
} as const;

const STATUS_LABELS = { online: 'browse.status.online', offline: 'browse.status.offline' } as const;

export function settingsPath(name: string): string {
  return `/admin/repository/repositories/${encodeURIComponent(name)}`;
}

function useFilters(): [ListFilters, (next: ListFilters) => void] {
  const [params, setParams] = useSearchParams();
  const filters = useMemo(() => readFilters(params), [params]);
  return [filters, (next) => setParams(writeFilters(next), { replace: true })];
}

function FacetGroup<T extends string>({
  title,
  entries,
  chosen,
  label,
  onToggle,
}: {
  title: string;
  entries: Array<[T, number]>;
  chosen: readonly T[];
  label: (value: T) => string;
  onToggle: (value: T) => void;
}) {
  if (entries.length === 0) {
    return null;
  }
  return (
    <fieldset className={styles.facet}>
      <legend>{title}</legend>
      {entries.map(([value, count]) => (
        <Text as="label" size="2" key={value} className={styles.facetOption}>
          <Checkbox checked={chosen.includes(value)} onCheckedChange={() => onToggle(value)} />
          <span className={styles.facetLabel}>{label(value)}</span>
          <Badge size="1" color="gray" variant="soft" radius="full">
            {count}
          </Badge>
        </Text>
      ))}
    </fieldset>
  );
}

function FilterPanel({
  items,
  filters,
  onChange,
}: {
  items: readonly RepositoryItem[];
  filters: ListFilters;
  onChange: (next: ListFilters) => void;
}) {
  const t = useT();
  const counts = useMemo(() => facetCounts(items, filters), [items, filters]);
  return (
    <div className={styles.filters}>
      <Button
        variant="outline"
        color="gray"
        size="2"
        disabled={activeFilterCount(filters) === 0 && filters.q === ''}
        onClick={() => onChange({ ...DEFAULT_FILTERS, sort: filters.sort, direction: filters.direction })}
      >
        <RotateCcw size={14} aria-hidden="true" />
        {t('browse.list.reset')}
      </Button>
      <fieldset className={styles.facet}>
        <legend>{t('browse.list.sortBy')}</legend>
        <Select.Root
          value={filters.sort}
          onValueChange={(sort) => onChange({ ...filters, sort: sort as SortKey })}
        >
          <Select.Trigger aria-label={t('browse.list.sortBy')} />
          <Select.Content>
            {(Object.keys(SORT_LABELS) as SortKey[]).map((key) => (
              <Select.Item key={key} value={key}>
                {t(SORT_LABELS[key])}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
        <Select.Root
          value={filters.direction}
          onValueChange={(direction) => onChange({ ...filters, direction: direction as Direction })}
        >
          <Select.Trigger aria-label={t('browse.list.direction')} />
          <Select.Content>
            <Select.Item value="asc">{t('browse.list.ascending')}</Select.Item>
            <Select.Item value="desc">{t('browse.list.descending')}</Select.Item>
          </Select.Content>
        </Select.Root>
      </fieldset>
      <FacetGroup
        title={t('browse.list.format')}
        entries={counts.formats}
        chosen={filters.formats}
        label={formatName}
        onToggle={(value) => onChange({ ...filters, formats: toggle(filters.formats, value) })}
      />
      <FacetGroup
        title={t('browse.list.type')}
        entries={counts.types}
        chosen={filters.types}
        label={(value) => t(TYPE_LABELS[value])}
        onToggle={(value) => onChange({ ...filters, types: toggle(filters.types, value) })}
      />
      <FacetGroup
        title={t('browse.list.status')}
        entries={counts.statuses}
        chosen={filters.statuses}
        label={(value) => t(STATUS_LABELS[value])}
        onToggle={(value) => onChange({ ...filters, statuses: toggle(filters.statuses, value) })}
      />
    </div>
  );
}

function shownCount(item: RepositoryItem, value: number | null): number | null {
  return item.type === 'group' ? null : value;
}

function relativeUrl(url: string): string {
  try {
    return new URL(url).pathname;
  } catch {
    return url;
  }
}

function Count({ value }: { value: number | null }) {
  const { language } = useI18n();
  return value === null ? <Text color="gray">—</Text> : <span>{formatNumber(language, value)}</span>;
}

function Size({ value }: { value: number | null }) {
  return value === null ? <Text color="gray">—</Text> : <FileSize bytes={value} />;
}

function SettingsLink({ item }: { item: RepositoryItem }) {
  const t = useT();
  if (!item.permissions.admin) {
    return null;
  }
  const label = t('browse.list.settingsOf', { name: item.name });
  return (
    <Tooltip content={label}>
      <IconButton asChild variant="ghost" color="gray" size="1">
        <Link to={settingsPath(item.name)} aria-label={label}>
          <Settings2 size={15} aria-hidden="true" />
        </Link>
      </IconButton>
    </Tooltip>
  );
}

function RepositoryCards({ items }: { items: RepositoryItem[] }) {
  const t = useT();
  return (
    <ul className={styles.cards} aria-label={t('browse.list.results')}>
      {items.map((item) => (
        <li key={item.name} className={styles.card}>
          <div className={styles.cardHead}>
            <Link to={browseUrl(item.name)} className={styles.name}>
              {item.name}
            </Link>
            <SettingsLink item={item} />
          </div>
          <Flex gap="2" align="center" wrap="wrap">
            <FormatBadge format={item.format} />
            <TypeBadge type={item.type} />
            <OnlineStatus online={item.online} />
          </Flex>
          <dl className={styles.stats}>
            <div>
              <dt>{t('browse.list.components')}</dt>
              <dd>
                <Count value={shownCount(item, item.componentCount)} />
              </dd>
            </div>
            <div>
              <dt>{t('browse.list.size')}</dt>
              <dd>
                <Size value={shownCount(item, item.totalSize)} />
              </dd>
            </div>
            <div>
              <dt>{t('browse.list.uploaded')}</dt>
              <dd>{item.lastUploaded ? <RelativeTime value={item.lastUploaded} /> : '—'}</dd>
            </div>
          </dl>
          {item.type === 'group' && item.members && (
            <Text size="1" color="gray">
              {t('browse.list.members', { count: item.members.length })}
            </Text>
          )}
          <UrlLine url={item.url} label={t('browse.list.copyUrlOf', { name: item.name })} />
        </li>
      ))}
    </ul>
  );
}

function SortHeader({
  column,
  filters,
  onChange,
  align,
}: {
  column: SortKey;
  filters: ListFilters;
  onChange: (next: ListFilters) => void;
  align?: 'right';
}) {
  const t = useT();
  const active = filters.sort === column;
  const Arrow = !active ? ArrowUpDown : filters.direction === 'asc' ? ArrowUp : ArrowDown;
  return (
    <Table.ColumnHeaderCell
      align={align}
      aria-sort={active ? (filters.direction === 'asc' ? 'ascending' : 'descending') : undefined}
    >
      <button
        type="button"
        className={styles.sort}
        onClick={() =>
          onChange({
            ...filters,
            sort: column,
            direction: active && filters.direction === 'asc' ? 'desc' : 'asc',
          })
        }
      >
        {t(SORT_LABELS[column])}
        <Arrow size={13} aria-hidden="true" opacity={active ? 1 : 0.4} />
      </button>
    </Table.ColumnHeaderCell>
  );
}

function RepositoryTable({
  items,
  filters,
  onChange,
}: {
  items: RepositoryItem[];
  filters: ListFilters;
  onChange: (next: ListFilters) => void;
}) {
  const t = useT();
  return (
    <div className={styles.tableWrap}>
      <Table.Root variant="ghost" size="2">
        <caption className="visually-hidden">{t('browse.list.results')}</caption>
        <Table.Header>
          <Table.Row>
            <SortHeader column="name" filters={filters} onChange={onChange} />
            <SortHeader column="format" filters={filters} onChange={onChange} />
            <SortHeader column="type" filters={filters} onChange={onChange} />
            <SortHeader column="status" filters={filters} onChange={onChange} />
            <SortHeader column="components" filters={filters} onChange={onChange} align="right" />
            <SortHeader column="size" filters={filters} onChange={onChange} align="right" />
            <SortHeader column="uploaded" filters={filters} onChange={onChange} />
            <Table.ColumnHeaderCell>{t('browse.list.url')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell width="44px">
              <span className="visually-hidden">{t('browse.list.actions')}</span>
            </Table.ColumnHeaderCell>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {items.map((item) => (
            <Table.Row key={item.name} align="center">
              <Table.RowHeaderCell>
                <Link to={browseUrl(item.name)} className={`${styles.name} ${styles.nowrap}`}>
                  {item.name}
                </Link>
              </Table.RowHeaderCell>
              <Table.Cell>
                <FormatBadge format={item.format} />
              </Table.Cell>
              <Table.Cell>
                <TypeBadge type={item.type} />
              </Table.Cell>
              <Table.Cell>
                <OnlineStatus online={item.online} />
              </Table.Cell>
              <Table.Cell align="right">
                <Count value={shownCount(item, item.componentCount)} />
              </Table.Cell>
              <Table.Cell align="right">
                <Size value={shownCount(item, item.totalSize)} />
              </Table.Cell>
              <Table.Cell>
                {item.lastUploaded ? <RelativeTime value={item.lastUploaded} /> : <Text color="gray">—</Text>}
              </Table.Cell>
              <Table.Cell className={styles.urlCell}>
                <UrlLine
                  url={item.url}
                  shown={relativeUrl(item.url)}
                  label={t('browse.list.copyUrlOf', { name: item.name })}
                />
              </Table.Cell>
              <Table.Cell align="right">
                <SettingsLink item={item} />
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table.Root>
    </div>
  );
}

export default function RepositoryListPage() {
  const t = useT();
  const { language } = useI18n();
  const phone = useMediaQuery(PHONE);
  const compact = useMediaQuery(COMPACT);
  const [filters, setFilters] = useFilters();
  const [view, setView] = useState<View>(() => (readStored(VIEW_KEY) === 'cards' ? 'cards' : 'table'));
  const [filtersOpen, setFiltersOpen] = useState(false);
  const query = useApi<RepositoryItem[]>(REPOSITORIES_PATH);
  const collator = useMemo(
    () => new Intl.Collator(language, { numeric: true, sensitivity: 'base' }),
    [language],
  );
  const items = query.data ?? [];
  const shown = useMemo(() => applyFilters(items, filters, collator), [items, filters, collator]);
  const shownView: View = phone ? 'cards' : view;
  const active = activeFilterCount(filters);

  const changeView = (next: string) => {
    const value: View = next === 'cards' ? 'cards' : 'table';
    writeStored(VIEW_KEY, value);
    setView(value);
  };

  const panel = <FilterPanel items={items} filters={filters} onChange={setFilters} />;

  return (
    <Page
      title={t('browse.list.title')}
      description={
        query.data ? t('browse.list.count', { count: items.length }) : t('browse.list.description')
      }
    >
      <div className={styles.layout}>
        {!compact && items.length > 0 && (
          <aside className={styles.aside} aria-label={t('browse.list.filters')}>
            {panel}
          </aside>
        )}
        <div className={styles.content}>
          <div className={styles.toolbar}>
            <TextField.Root
              className={styles.filterText}
              type="search"
              value={filters.q}
              placeholder={t('browse.list.filterPlaceholder')}
              aria-label={t('browse.list.filterLabel')}
              onChange={(event) => setFilters({ ...filters, q: event.target.value })}
            >
              <TextField.Slot>
                <Search size={15} aria-hidden="true" />
              </TextField.Slot>
              {filters.q !== '' && (
                <TextField.Slot>
                  <IconButton
                    size="1"
                    variant="ghost"
                    color="gray"
                    aria-label={t('browse.list.clearFilter')}
                    onClick={() => setFilters({ ...filters, q: '' })}
                  >
                    <X size={14} aria-hidden="true" />
                  </IconButton>
                </TextField.Slot>
              )}
            </TextField.Root>
            {compact && items.length > 0 && (
              <Dialog.Root open={filtersOpen} onOpenChange={setFiltersOpen}>
                <Dialog.Trigger>
                  <Button variant="soft" color="gray">
                    <ListFilter size={15} aria-hidden="true" />
                    {active > 0
                      ? t('browse.list.filtersActive', { count: active })
                      : t('browse.list.filters')}
                  </Button>
                </Dialog.Trigger>
                <Dialog.Content maxWidth="420px">
                  <Flex justify="between" align="center" mb="3">
                    <Dialog.Title mb="0">{t('browse.list.filters')}</Dialog.Title>
                    <Dialog.Close>
                      <IconButton variant="ghost" color="gray" aria-label={t('browse.list.closeFilters')}>
                        <X size={18} aria-hidden="true" />
                      </IconButton>
                    </Dialog.Close>
                  </Flex>
                  <Dialog.Description className="visually-hidden">
                    {t('browse.list.filtersDescription')}
                  </Dialog.Description>
                  {panel}
                  <Flex justify="end" mt="4">
                    <Dialog.Close>
                      <Button>{t('browse.list.showResults', { count: shown.length })}</Button>
                    </Dialog.Close>
                  </Flex>
                </Dialog.Content>
              </Dialog.Root>
            )}
            {!phone && (
              <SegmentedControl.Root
                value={view}
                onValueChange={changeView}
                aria-label={t('browse.list.view')}
              >
                <SegmentedControl.Item value="table" aria-label={t('browse.list.viewTable')}>
                  <Flex align="center" gap="1">
                    <Rows3 size={15} aria-hidden="true" />
                    <span className={styles.viewText}>{t('browse.list.viewTable')}</span>
                  </Flex>
                </SegmentedControl.Item>
                <SegmentedControl.Item value="cards" aria-label={t('browse.list.viewCards')}>
                  <Flex align="center" gap="1">
                    <LayoutGrid size={15} aria-hidden="true" />
                    <span className={styles.viewText}>{t('browse.list.viewCards')}</span>
                  </Flex>
                </SegmentedControl.Item>
              </SegmentedControl.Root>
            )}
          </div>
          {query.isPending ? (
            <div className={styles.loading} aria-busy="true">
              {[0, 1, 2, 3].map((index) => (
                <Skeleton key={index} height="56px" />
              ))}
            </div>
          ) : query.isError ? (
            <ErrorState error={query.error} onRetry={() => query.refetch()} />
          ) : items.length === 0 ? (
            <EmptyState
              icon={FolderOpen}
              title={t('browse.list.emptyTitle')}
              description={t('browse.list.emptyText')}
            />
          ) : shown.length === 0 ? (
            <EmptyState
              icon={Search}
              title={t('browse.list.noMatches')}
              action={
                <Button variant="soft" onClick={() => setFilters({ ...DEFAULT_FILTERS })}>
                  {t('browse.list.reset')}
                </Button>
              }
            />
          ) : (
            <>
              <Text as="p" size="2" color="gray" role="status">
                {t('browse.list.shown', { count: shown.length, total: items.length })}
              </Text>
              {shownView === 'cards' ? (
                <RepositoryCards items={shown} />
              ) : (
                <RepositoryTable items={shown} filters={filters} onChange={setFilters} />
              )}
            </>
          )}
        </div>
      </div>
    </Page>
  );
}
