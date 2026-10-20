import {
  Badge,
  Button,
  Checkbox,
  Dialog,
  DropdownMenu,
  Flex,
  Heading,
  IconButton,
  Select,
  Skeleton,
  Table,
  Text,
  TextField,
} from '@radix-ui/themes';
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  ChevronDown,
  ChevronUp,
  Copy,
  Ellipsis,
  FolderOpen,
  RotateCcw,
  Search,
  SlidersHorizontal,
  X,
} from 'lucide-react';
import { type ReactNode, useMemo, useRef, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import { useApi } from '../../../api';
import { formatNumber, useI18n, useT } from '../../../i18n';
import {
  COMPACT,
  CopyButton,
  copyText,
  EmptyState,
  ErrorState,
  FormatBadge,
  formatName,
  PHONE,
  TypeBadge,
  useDocumentTitle,
  useMediaQuery,
  useToast,
} from '../../../ui';
import { Pager, pageCount, pageSlice } from '../Pager';
import { OnlineStatus } from '../parts';
import { browseUrl } from '../tree/model';
import { REPOSITORIES_PATH, type RepositoryItem } from '../types';
import {
  applyFilters,
  DEFAULT_FILTERS,
  type Direction,
  facetCounts,
  type ListFilters,
  readFilters,
  SORT_KEYS,
  type SortKey,
  toggle,
  writeFilters,
} from './filters';
import styles from './RepositoryList.module.css';

const SORT_LABELS: Record<SortKey, string> = {
  name: 'browse.list.sort.name',
  format: 'browse.list.sort.format',
  type: 'browse.list.sort.type',
  status: 'browse.list.sort.status',
};

const TYPE_LABELS = {
  hosted: 'browse.list.type.hosted',
  proxy: 'browse.list.type.proxy',
  group: 'browse.list.type.group',
} as const;

const STATUS_LABELS = { online: 'browse.status.online', offline: 'browse.status.offline' } as const;

const FORMATS_SHOWN = 7;
const DEFAULT_PAGE_SIZE = 40;

export function settingsPath(name: string): string {
  return `/admin/repository/repositories/${encodeURIComponent(name)}`;
}

type FilterChange = (patch: Partial<ListFilters>) => void;

function useFilters(): [ListFilters, FilterChange] {
  const [params, setParams] = useSearchParams();
  const filters = useMemo(() => readFilters(params), [params]);
  const latest = useRef(filters);
  latest.current = filters;
  const change: FilterChange = (patch) => {
    latest.current = { ...latest.current, ...patch };
    setParams(writeFilters(latest.current), { replace: true });
  };
  return [filters, change];
}

function FacetGroup<T extends string>({
  title,
  entries,
  chosen,
  label,
  onToggle,
  limit,
}: {
  title: string;
  entries: Array<[T, number]>;
  chosen: readonly T[];
  label: (value: T) => string;
  onToggle: (value: T) => void;
  limit?: number;
}) {
  const t = useT();
  const [all, setAll] = useState(false);
  if (entries.length === 0) {
    return null;
  }
  const folded = limit !== undefined && entries.length > limit;
  const shown = folded && !all ? entries.slice(0, limit) : entries;
  return (
    <fieldset className={styles.facet}>
      <legend>{title}</legend>
      {shown.map(([value, count]) => (
        <Text as="label" size="2" key={value} className={styles.facetOption}>
          <Checkbox checked={chosen.includes(value)} onCheckedChange={() => onToggle(value)} />
          <span className={styles.facetLabel}>{label(value)}</span>
          <Badge size="1" color="gray" variant="soft" radius="full">
            {count}
          </Badge>
        </Text>
      ))}
      {folded && (
        <Button variant="ghost" size="1" className={styles.more} onClick={() => setAll(!all)}>
          {all ? <ChevronUp size={12} aria-hidden="true" /> : <ChevronDown size={12} aria-hidden="true" />}
          {all ? t('browse.list.seeLess') : t('browse.list.seeMore')}
        </Button>
      )}
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
  onChange: FilterChange;
}) {
  const t = useT();
  const { language } = useI18n();
  const counts = useMemo(() => facetCounts(items, filters), [items, filters]);
  const collator = useMemo(() => new Intl.Collator(language, { sensitivity: 'base' }), [language]);
  const formats = [...counts.formats].sort(([a], [b]) => collator.compare(formatName(a), formatName(b)));
  const types = [...counts.types].sort(([a], [b]) => collator.compare(t(TYPE_LABELS[a]), t(TYPE_LABELS[b])));
  return (
    <div className={styles.filters}>
      <Button
        variant="outline"
        color="gray"
        size="2"
        className={styles.reset}
        onClick={() => onChange({ ...DEFAULT_FILTERS })}
      >
        <RotateCcw size={14} aria-hidden="true" />
        {t('browse.list.reset')}
      </Button>
      <fieldset className={`${styles.facet} ${styles.sortFacet}`}>
        <legend>{t('browse.list.sortBy')}</legend>
        <Select.Root value={filters.sort} onValueChange={(sort) => onChange({ sort: sort as SortKey })}>
          <Select.Trigger aria-label={t('browse.list.sortBy')} />
          <Select.Content>
            {SORT_KEYS.map((key) => (
              <Select.Item key={key} value={key}>
                {t(SORT_LABELS[key])}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
        <Select.Root
          value={filters.direction}
          onValueChange={(direction) => onChange({ direction: direction as Direction })}
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
        entries={formats}
        chosen={filters.formats}
        label={formatName}
        limit={FORMATS_SHOWN}
        onToggle={(value) => onChange({ formats: toggle(filters.formats, value) })}
      />
      <FacetGroup
        title={t('browse.list.type')}
        entries={types}
        chosen={filters.types}
        label={(value) => t(TYPE_LABELS[value])}
        onToggle={(value) => onChange({ types: toggle(filters.types, value) })}
      />
      <FacetGroup
        title={t('browse.list.status')}
        entries={counts.statuses}
        chosen={filters.statuses}
        label={(value) => t(STATUS_LABELS[value])}
        onToggle={(value) => onChange({ statuses: toggle(filters.statuses, value) })}
      />
    </div>
  );
}

export function repositoryPath(url: string): string {
  let path = url;
  try {
    path = new URL(url).pathname;
  } catch {
    path = url;
  }
  return path.endsWith('/') ? path : `${path}/`;
}

function withSlash(url: string): string {
  return url.endsWith('/') ? url : `${url}/`;
}

function RowMenu({ item }: { item: RepositoryItem }) {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const copy = async () => {
    try {
      await copyText(withSlash(item.url));
      toast.success(t('browse.list.urlCopied'));
    } catch {
      toast.error(t('browse.list.urlCopyFailed'));
    }
  };
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <IconButton
          variant="ghost"
          color="gray"
          size="1"
          aria-label={t('browse.list.rowActions', { name: item.name })}
          onClick={(event) => event.stopPropagation()}
        >
          <Ellipsis size={16} aria-hidden="true" />
        </IconButton>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end" onClick={(event) => event.stopPropagation()}>
        <DropdownMenu.Item onSelect={() => navigate(browseUrl(item.name))}>
          {t('browse.list.open')}
        </DropdownMenu.Item>
        <DropdownMenu.Item onSelect={copy}>{t('browse.list.copyUrl')}</DropdownMenu.Item>
        {item.permissions.admin && (
          <DropdownMenu.Item onSelect={() => navigate(settingsPath(item.name))}>
            {t('browse.list.settings')}
          </DropdownMenu.Item>
        )}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

function RepositoryCards({ items }: { items: RepositoryItem[] }) {
  const t = useT();
  const toast = useToast();
  const copy = async (item: RepositoryItem) => {
    try {
      await copyText(withSlash(item.url));
      toast.success(t('browse.list.urlCopied'));
    } catch {
      toast.error(t('browse.list.urlCopyFailed'));
    }
  };
  return (
    <ul className={styles.cards} aria-label={t('browse.list.results')}>
      {items.map((item) => (
        <li key={item.name} className={styles.card}>
          <div className={styles.cardMain}>
            <div className={styles.cardHead}>
              <span
                className={styles.cardDot}
                data-online={item.online}
                role="img"
                aria-label={item.online ? t('browse.status.online') : t('browse.status.offline')}
              />
              <Link to={browseUrl(item.name)} className={styles.cardName}>
                {item.name}
              </Link>
            </div>
            <Flex gap="2" align="center" wrap="wrap">
              <FormatBadge format={item.format} />
              <TypeBadge type={item.type} />
            </Flex>
          </div>
          <IconButton
            variant="outline"
            color="gray"
            size="4"
            className={styles.cardCopy}
            aria-label={t('browse.list.copyUrlOf', { name: item.name })}
            onClick={() => copy(item)}
          >
            <Copy size={16} aria-hidden="true" />
          </IconButton>
        </li>
      ))}
    </ul>
  );
}

function SortHeader({
  column,
  filters,
  onChange,
}: {
  column: SortKey;
  filters: ListFilters;
  onChange: FilterChange;
}) {
  const t = useT();
  const active = filters.sort === column;
  const Arrow = !active ? ArrowUpDown : filters.direction === 'asc' ? ArrowUp : ArrowDown;
  return (
    <Table.ColumnHeaderCell
      aria-sort={active ? (filters.direction === 'asc' ? 'ascending' : 'descending') : undefined}
    >
      <button
        type="button"
        className={styles.sort}
        onClick={() =>
          onChange({
            sort: column,
            direction: active && filters.direction === 'asc' ? 'desc' : 'asc',
          })
        }
      >
        {t(SORT_LABELS[column])}
        <Arrow size={14} aria-hidden="true" opacity={active ? 1 : 0.6} />
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
  onChange: FilterChange;
}) {
  const t = useT();
  const navigate = useNavigate();
  return (
    <Table.Root variant="surface" size="2" className={styles.table}>
      <caption className="visually-hidden">{t('browse.list.results')}</caption>
      <Table.Header>
        <Table.Row>
          <SortHeader column="name" filters={filters} onChange={onChange} />
          <SortHeader column="format" filters={filters} onChange={onChange} />
          <SortHeader column="type" filters={filters} onChange={onChange} />
          <SortHeader column="status" filters={filters} onChange={onChange} />
          <Table.ColumnHeaderCell>{t('browse.list.url')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell width="56px">
            <span className="visually-hidden">{t('browse.list.actions')}</span>
          </Table.ColumnHeaderCell>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {items.map((item) => (
          <Table.Row
            key={item.name}
            align="center"
            className={styles.row}
            onClick={(event) => {
              if (!(event.target as HTMLElement).closest('a, button, [role=menu]')) {
                navigate(browseUrl(item.name));
              }
            }}
          >
            <Table.RowHeaderCell>
              <Link to={browseUrl(item.name)} className={styles.name}>
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
            <Table.Cell className={styles.urlCell}>
              <span className={styles.url}>
                <span className={styles.urlText} title={withSlash(item.url)}>
                  {repositoryPath(item.url)}
                </span>
                <CopyButton text={withSlash(item.url)} label={t('browse.list.copyUrl')} />
              </span>
            </Table.Cell>
            <Table.Cell align="right">
              <RowMenu item={item} />
            </Table.Cell>
          </Table.Row>
        ))}
      </Table.Body>
    </Table.Root>
  );
}

function Title({ count }: { count: ReactNode }) {
  const t = useT();
  return (
    <div className={styles.titleRow}>
      <Heading as="h1" size="7" weight="bold" className={styles.title}>
        {t('browse.list.title')}
      </Heading>
      {count}
    </div>
  );
}

export default function RepositoryListPage() {
  const t = useT();
  const { language } = useI18n();
  useDocumentTitle(t('browse.list.title'));
  const phone = useMediaQuery(PHONE);
  const compact = useMediaQuery(COMPACT);
  const [filters, setFilters] = useFilters();
  const [filtersOpen, setFiltersOpen] = useState(false);
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);
  const query = useApi<RepositoryItem[]>(REPOSITORIES_PATH);
  const collator = useMemo(
    () => new Intl.Collator(language, { numeric: true, sensitivity: 'base' }),
    [language],
  );
  const items = query.data ?? [];
  const shown = useMemo(() => applyFilters(items, filters, collator), [items, filters, collator]);
  const lastPage = pageCount(shown.length, pageSize) - 1;
  const currentPage = Math.min(page, lastPage);
  const visible = pageSlice(shown, currentPage, pageSize);
  const change: FilterChange = (patch) => {
    setPage(0);
    setFilters(patch);
  };

  const panel = <FilterPanel items={items} filters={filters} onChange={change} />;
  const count = query.data ? (
    <Text size="2" color="gray">
      {t('browse.list.total', { count: formatNumber(language, items.length) })}
    </Text>
  ) : null;

  return (
    <div className={styles.page}>
      <Title count={count} />
      <div className={styles.layout}>
        {!compact && (
          <aside className={styles.aside} aria-label={t('browse.list.filters')}>
            {panel}
          </aside>
        )}
        <div className={styles.content}>
          <div className={styles.toolbar}>
            <TextField.Root
              className={styles.filterText}
              size="3"
              type="search"
              value={filters.q}
              placeholder={t('browse.list.filterPlaceholder')}
              aria-label={t('browse.list.filterLabel')}
              onChange={(event) => change({ q: event.target.value })}
            >
              <TextField.Slot>
                <Search size={16} aria-hidden="true" />
              </TextField.Slot>
              {filters.q !== '' && (
                <TextField.Slot>
                  <IconButton
                    size="1"
                    variant="ghost"
                    color="gray"
                    aria-label={t('browse.list.clearFilter')}
                    onClick={() => change({ q: '' })}
                  >
                    <X size={14} aria-hidden="true" />
                  </IconButton>
                </TextField.Slot>
              )}
            </TextField.Root>
            {compact && (
              <Dialog.Root open={filtersOpen} onOpenChange={setFiltersOpen}>
                <Dialog.Trigger>
                  <Button variant="outline" color="gray" size="3">
                    <SlidersHorizontal size={16} aria-hidden="true" />
                    {t('browse.list.filters')}
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
                </Dialog.Content>
              </Dialog.Root>
            )}
          </div>
          {query.isPending ? (
            <div className={styles.loading} aria-busy="true">
              {[0, 1, 2, 3].map((index) => (
                <Skeleton key={index} height="45px" />
              ))}
            </div>
          ) : query.isError ? (
            <ErrorState error={query.error} onRetry={() => query.refetch()} />
          ) : items.length === 0 ? (
            <EmptyState icon={FolderOpen} title={t('browse.list.emptyText')} />
          ) : shown.length === 0 ? (
            <EmptyState icon={Search} title={t('browse.list.noMatches')} />
          ) : (
            <>
              {phone ? (
                <RepositoryCards items={visible} />
              ) : (
                <RepositoryTable items={visible} filters={filters} onChange={change} />
              )}
              <Pager
                total={shown.length}
                page={currentPage}
                size={pageSize}
                onPage={setPage}
                onSize={(size) => {
                  setPage(0);
                  setPageSize(size);
                }}
              />
            </>
          )}
        </div>
      </div>
    </div>
  );
}
