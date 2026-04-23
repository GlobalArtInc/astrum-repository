import {
  Badge,
  Button,
  Card,
  Code,
  Heading,
  IconButton,
  Inset,
  TextField as RadixTextField,
  Select,
  Table,
  Text,
} from '@radix-ui/themes';
import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { ChevronDown, ChevronUp, Funnel, RefreshCw, Search, X } from 'lucide-react';
import { Fragment, type ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import { useSearchParams } from 'react-router';
import { type ApiError, api, useApi } from '../../api';
import { useI18n } from '../../i18n';
import {
  CheckList,
  describeError,
  FilterLayout,
  FilterSection,
  NARROW,
  ResetFiltersButton,
  useDocumentTitle,
  useMediaQuery,
} from '../../ui';
import styles from './Audit.module.css';
import {
  type AuditEvent,
  CATEGORIES,
  CATEGORY_COLORS,
  CATEGORY_LABELS,
  categoryOf,
  EVENT_TYPE_LABELS,
  EVENT_TYPES,
  kindLabel,
  summaryOf,
} from './events';
import { Pager } from './Pager';
import {
  type AuditFilters,
  type AuditPage as AuditPageData,
  type AuditPaging,
  auditLogPath,
  EMPTY_FILTERS,
  type EventType,
  PERIODS,
  type Period,
  readFilters,
  readPaging,
  writeParams,
} from './query';

const ALL = '*';
const DEBOUNCE_MS = 300;
const REPOSITORIES = '/service/rest/v1/repositories';

interface RepositoryOption {
  name: string;
  format: string;
}

const PERIOD_LABELS: Record<Period, string> = {
  1: 'audit.filters.last1',
  7: 'audit.filters.last7',
  30: 'audit.filters.last30',
  90: 'audit.filters.last90',
};

function useDebounced(value: string, apply: (value: string) => void) {
  const [text, setText] = useState(value);
  const latest = useRef(apply);
  latest.current = apply;
  useEffect(() => {
    setText(value);
  }, [value]);
  useEffect(() => {
    if (text === value) {
      return;
    }
    const timer = window.setTimeout(() => latest.current(text), DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [text, value]);
  return [text, setText] as const;
}

function EventDetails({ event }: { event: AuditEvent }) {
  const { t } = useI18n();
  const attributes = event.attributes && Object.keys(event.attributes).length > 0 ? event.attributes : null;
  return (
    <div className={styles.details}>
      <Heading as="h3" size="3" className={styles.detailsTitle}>
        {t('audit.details.title')}
      </Heading>
      <div className={styles.detailLines}>
        <Text as="span" size="2">
          <strong>{t('audit.details.domain')}</strong> {event.domain}
        </Text>
        <Text as="span" size="2">
          <strong>{t('audit.details.type')}</strong> {event.type}
        </Text>
        <Text as="span" size="2">
          <strong>{t('audit.details.context')}</strong> {event.context || '-'}
        </Text>
      </div>
      {attributes && (
        <>
          <Text as="span" size="2" weight="medium">
            {t('audit.details.attributes')}
          </Text>
          <Code size="2" className={styles.attributes}>
            {JSON.stringify(attributes, null, 2)}
          </Code>
        </>
      )}
    </div>
  );
}

export default function AuditPage() {
  const { t, language } = useI18n();
  useDocumentTitle(t('audit.page.title'));
  const narrow = useMediaQuery(NARROW);
  const [params, setParams] = useSearchParams();
  const filters = readFilters(params);
  const paging = readPaging(params);
  const [now, setNow] = useState(() => new Date());
  const [sheet, setSheet] = useState(false);
  const [open, setOpen] = useState<number | null>(null);
  const repositories = useApi<RepositoryOption[]>(REPOSITORIES);

  const update = (patch: Partial<AuditFilters>, nextPaging: Partial<AuditPaging> = { page: 1 }) => {
    setParams(
      (previous) =>
        writeParams({ ...readFilters(previous), ...patch }, { ...readPaging(previous), ...nextPaging }),
      { replace: true },
    );
  };

  const [search, setSearch] = useDebounced(filters.q, (q) => update({ q }));
  const [initiator, setInitiator] = useDebounced(filters.initiator, (value) => update({ initiator: value }));

  const path = auditLogPath(filters, paging, now);
  const query = useQuery<AuditPageData<AuditEvent>, ApiError>({
    queryKey: ['api', path],
    queryFn: ({ signal }) => api.get<AuditPageData<AuditEvent>>(path, { signal }),
    placeholderData: keepPreviousData,
  });

  const formats = useMemo(
    () => [...new Set((repositories.data ?? []).map((repository) => repository.format))].sort(),
    [repositories.data],
  );
  const repositoryNames = useMemo(() => {
    const names = (repositories.data ?? [])
      .filter((repository) => filters.format === '' || repository.format === filters.format)
      .map((repository) => repository.name)
      .sort((left, right) => left.localeCompare(right));
    if (filters.repository !== '' && !names.includes(filters.repository)) {
      names.push(filters.repository);
    }
    return names;
  }, [repositories.data, filters.format, filters.repository]);

  const time = new Intl.DateTimeFormat(language, {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });

  const reset = () => {
    setSearch('');
    setInitiator('');
    setParams(writeParams(EMPTY_FILTERS, { page: 1, size: paging.size }), { replace: true });
  };

  const filterColumn = (
    <div className={styles.filters}>
      <ResetFiltersButton onClick={reset} />
      <div className={styles.filterSections}>
        <FilterSection title={t('audit.filters.period')}>
          <Select.Root
            value={String(filters.period)}
            onValueChange={(value) => update({ period: Number(value) as Period })}
          >
            <Select.Trigger className={styles.wide} aria-label={t('audit.filters.period')} />
            <Select.Content position="popper">
              {PERIODS.map((days) => (
                <Select.Item key={days} value={String(days)}>
                  {t(PERIOD_LABELS[days])}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </FilterSection>
        <FilterSection title={t('audit.filters.categories')}>
          <CheckList
            options={CATEGORIES.map((category) => ({ value: category, label: t(CATEGORY_LABELS[category]) }))}
            selected={filters.categories}
            onChange={(categories) => update({ categories: categories as AuditFilters['categories'] })}
          />
        </FilterSection>
        <FilterSection title={t('audit.filters.format')}>
          <Select.Root
            value={filters.format || ALL}
            onValueChange={(value) => {
              const format = value === ALL ? '' : value;
              update({ format, repository: '' });
            }}
          >
            <Select.Trigger className={styles.wide} aria-label={t('audit.filters.format')} />
            <Select.Content position="popper">
              <Select.Item value={ALL}>{t('audit.filters.allFormats')}</Select.Item>
              {formats.map((format) => (
                <Select.Item key={format} value={format}>
                  {format.toUpperCase()}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </FilterSection>
        <FilterSection title={t('audit.filters.repository')}>
          <Select.Root
            value={filters.repository || ALL}
            onValueChange={(value) => update({ repository: value === ALL ? '' : value })}
          >
            <Select.Trigger className={styles.wide} aria-label={t('audit.filters.repository')} />
            <Select.Content position="popper">
              <Select.Item value={ALL}>{t('audit.filters.allRepositories')}</Select.Item>
              {repositoryNames.map((name) => (
                <Select.Item key={name} value={name}>
                  {name}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </FilterSection>
        <FilterSection title={t('audit.filters.types')}>
          <CheckList
            options={EVENT_TYPES.map((type) => ({ value: type, label: t(EVENT_TYPE_LABELS[type]) }))}
            selected={filters.types}
            onChange={(types) => update({ types: types as EventType[] })}
          />
        </FilterSection>
        <FilterSection title={t('audit.filters.initiator')}>
          <RadixTextField.Root
            value={initiator}
            placeholder={t('audit.filters.initiatorPlaceholder')}
            aria-label={t('audit.filters.initiator')}
            onChange={(event) => setInitiator(event.target.value)}
          />
        </FilterSection>
      </div>
    </div>
  );

  const data = query.data;
  const total = data?.pagination.totalItems ?? 0;
  const pages = data?.pagination.totalPages ?? 0;

  let table: ReactNode;
  if (query.isError && !data) {
    table = (
      <Card size="1" className={styles.message} role="alert">
        <Text size="2">{describeError(t, query.error)}</Text>
      </Card>
    );
  } else if (!data) {
    table = (
      <Card size="1" className={styles.message} role="status">
        <Text size="2" className={styles.gray}>
          {t('audit.table.loading')}
        </Text>
      </Card>
    );
  } else if (data.items.length === 0) {
    const filtered =
      filters.categories.length > 0 ||
      filters.types.length > 0 ||
      filters.repository !== '' ||
      filters.initiator !== '' ||
      filters.q !== '';
    table = (
      <Card size="1" className={styles.message}>
        <Text as="p" size="3" weight="bold">
          {t('audit.table.emptyTitle')}
        </Text>
        <Text as="p" size="2" className={styles.gray}>
          {filtered ? t('audit.table.noMatches') : t('audit.table.emptyText')}
        </Text>
      </Card>
    );
  } else {
    table = (
      <Card size="1" className={styles.card}>
        <Inset clip="border-box" side="all">
          <div className={styles.scroll}>
            <Table.Root variant="ghost" size="2" aria-label={t('audit.table.label')}>
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell
                    className={styles.toggleCell}
                    aria-label={t('audit.table.expand')}
                  />
                  <Table.ColumnHeaderCell>{t('audit.table.time')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('audit.table.category')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('audit.table.event')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('audit.table.summary')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('audit.table.initiator')}</Table.ColumnHeaderCell>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {data.items.map((event) => {
                  const category = categoryOf(event);
                  const expanded = open === event.id;
                  const toggle = () => setOpen(expanded ? null : event.id);
                  return (
                    <Fragment key={event.id}>
                      <Table.Row className={styles.row} onClick={toggle}>
                        <Table.Cell className={styles.toggleCell}>
                          <button
                            type="button"
                            className={styles.toggle}
                            aria-expanded={expanded}
                            aria-label={expanded ? t('audit.table.collapse') : t('audit.table.expand')}
                            onClick={(click) => {
                              click.stopPropagation();
                              toggle();
                            }}
                          >
                            {expanded ? (
                              <ChevronUp size={16} aria-hidden="true" />
                            ) : (
                              <ChevronDown size={16} aria-hidden="true" />
                            )}
                          </button>
                        </Table.Cell>
                        <Table.Cell>
                          <Text size="2" className={styles.nowrap}>
                            <time dateTime={event.timestamp}>{time.format(new Date(event.timestamp))}</time>
                          </Text>
                        </Table.Cell>
                        <Table.Cell>
                          <Badge size="1" color={CATEGORY_COLORS[category]} className={styles.badge}>
                            {t(CATEGORY_LABELS[category])}
                          </Badge>
                        </Table.Cell>
                        <Table.Cell>
                          <Text size="2" weight="medium">
                            {kindLabel(t, event)}
                          </Text>
                        </Table.Cell>
                        <Table.Cell>
                          <Text size="2">{summaryOf(t, event)}</Text>
                        </Table.Cell>
                        <Table.Cell>
                          <Text size="2" className={styles.gray}>
                            {event.initiator}
                          </Text>
                        </Table.Cell>
                      </Table.Row>
                      {expanded && (
                        <Table.Row>
                          <Table.Cell colSpan={6} className={styles.detailsCell}>
                            <EventDetails event={event} />
                          </Table.Cell>
                        </Table.Row>
                      )}
                    </Fragment>
                  );
                })}
              </Table.Body>
            </Table.Root>
          </div>
        </Inset>
      </Card>
    );
  }

  return (
    <div className={styles.page}>
      <FilterLayout
        label={t('audit.filters.label')}
        contentLabel={t('audit.page.contentLabel')}
        width={280}
        gap={16}
        filters={filterColumn}
        sheetOpen={sheet}
        onSheetOpenChange={setSheet}
        onResetAll={reset}
      >
        <div className={styles.head}>
          <div className={styles.heading}>
            <Heading as="h1" size="6" weight="bold">
              {t('audit.page.title')}
            </Heading>
            <Text as="p" size="2" className={styles.gray}>
              {data ? t('audit.page.count', { count: total }) : ' '}
            </Text>
          </div>
          <Button
            type="button"
            variant="ghost"
            onClick={() => {
              setNow(new Date());
              void query.refetch();
            }}
          >
            <RefreshCw size={16} aria-hidden="true" />
            {t('audit.page.refresh')}
          </Button>
        </div>
        <div className={styles.toolbar} role="toolbar" aria-label={t('audit.page.toolbar')}>
          <RadixTextField.Root
            className={styles.search}
            value={search}
            placeholder={t('audit.page.searchPlaceholder')}
            aria-label={t('audit.page.search')}
            onChange={(event) => setSearch(event.target.value)}
          >
            <RadixTextField.Slot>
              <Search size={14} aria-hidden="true" />
            </RadixTextField.Slot>
            {search !== '' && (
              <RadixTextField.Slot side="right">
                <IconButton
                  type="button"
                  size="1"
                  variant="ghost"
                  color="gray"
                  aria-label={t('audit.page.clearSearch')}
                  onClick={() => {
                    setSearch('');
                    update({ q: '' });
                  }}
                >
                  <X size={14} aria-hidden="true" />
                </IconButton>
              </RadixTextField.Slot>
            )}
          </RadixTextField.Root>
          {narrow && (
            <Button
              type="button"
              variant="outline"
              color="gray"
              aria-label={t('audit.page.openFilters')}
              onClick={() => setSheet(true)}
            >
              <Funnel size={16} aria-hidden="true" />
              {t('audit.page.filterButton')}
            </Button>
          )}
        </div>
        {table}
        {data && total > 0 && (
          <Pager
            page={paging.page}
            pages={pages}
            total={total}
            size={paging.size}
            onPage={(page) => update({}, { page })}
            onSize={(size) => update({}, { page: 1, size })}
          />
        )}
      </FilterLayout>
    </div>
  );
}
