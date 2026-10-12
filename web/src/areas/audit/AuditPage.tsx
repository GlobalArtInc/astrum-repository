import { Button, TextField as RadixTextField, Text } from '@radix-ui/themes';
import { useInfiniteQuery } from '@tanstack/react-query';
import { ListFilter, RotateCw, ScrollText, Search, SearchX } from 'lucide-react';
import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { useSearchParams } from 'react-router';
import { type ApiError, api, useApi } from '../../api';
import { type Translate, useT } from '../../i18n';
import {
  type Column,
  DataTable,
  DateTime,
  EmptyState,
  FieldShell,
  Page,
  PHONE,
  SelectField,
  useMediaQuery,
} from '../../ui';
import styles from './Audit.module.css';
import {
  AUDIT_FACETS,
  type AuditEvent,
  type AuditFacets,
  type AuditFilters,
  attributeEntries,
  auditPath,
  DOMAIN_LABELS,
  EMPTY_FILTERS,
  isFiltered,
  knownDomains,
  knownTypes,
  nextBefore,
  readFilters,
  TYPE_LABELS,
  writeFilters,
} from './query';

const ALL = '*';
const DEBOUNCE_MS = 300;
const REPOSITORIES = '/service/rest/v1/repositories';

const option = (value: string, label: string) => ({ value, label });

export function domainLabel(t: Translate, domain: string): string {
  const key = DOMAIN_LABELS[domain];
  return key ? t(key) : domain;
}

export function typeLabel(t: Translate, type: string): string {
  const key = TYPE_LABELS[type];
  return key ? t(key) : type;
}

function Attributes({ event }: { event: AuditEvent }) {
  const t = useT();
  const entries = attributeEntries(event.attributes);
  if (entries.length === 0) {
    return (
      <>
        <span aria-hidden="true">—</span>
        <span className="visually-hidden">{t('audit.table.noAttributes')}</span>
      </>
    );
  }
  return (
    <details className={styles.details}>
      <summary>{t('audit.table.attributeCount', { count: entries.length })}</summary>
      <dl className={styles.attributes}>
        {entries.map(([key, value]) => (
          <div key={key} className={styles.attribute}>
            <dt>{key}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </details>
  );
}

function DateField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange(value: string): void;
}) {
  const id = useId();
  return (
    <FieldShell id={id} label={label}>
      <RadixTextField.Root
        id={id}
        type="date"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </FieldShell>
  );
}

export default function AuditPage() {
  const t = useT();
  const searchId = useId();
  const phone = useMediaQuery(PHONE);
  const [params, setParams] = useSearchParams();
  const filters = readFilters(params);
  const [text, setText] = useState(filters.q);
  const pushed = useRef(filters.q);
  const [open, setOpen] = useState(false);
  const facets = useApi<AuditFacets>(AUDIT_FACETS);
  const repositories = useApi<Array<{ name: string }>>(REPOSITORIES);

  const update = (patch: Partial<AuditFilters>) => {
    setParams((previous) => writeFilters({ ...readFilters(previous), ...patch }), { replace: true });
  };

  useEffect(() => {
    if (filters.q !== pushed.current) {
      pushed.current = filters.q;
      setText(filters.q);
    }
  }, [filters.q]);

  useEffect(() => {
    if (text === pushed.current) {
      return;
    }
    const timer = window.setTimeout(() => {
      pushed.current = text;
      setParams((previous) => writeFilters({ ...readFilters(previous), q: text }), { replace: true });
    }, DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [text, setParams]);

  const path = auditPath(filters);
  const query = useInfiniteQuery<
    AuditEvent[],
    ApiError,
    { pages: AuditEvent[][] },
    readonly unknown[],
    number | null
  >({
    queryKey: ['api', path, 'pages'],
    queryFn: ({ pageParam, signal }) => api.get<AuditEvent[]>(auditPath(filters, pageParam), { signal }),
    initialPageParam: null,
    getNextPageParam: (last) => nextBefore(last),
    staleTime: 10_000,
  });
  const events = useMemo(() => query.data?.pages.flat(), [query.data]);

  const domainOptions = useMemo(
    () =>
      knownDomains(facets.data?.domains)
        .map((domain) => option(domain, domainLabel(t, domain)))
        .sort((left, right) => left.label.localeCompare(right.label)),
    [facets.data, t],
  );
  const typeOptions = useMemo(
    () => knownTypes(facets.data?.types).map((type) => option(type, typeLabel(t, type))),
    [facets.data, t],
  );
  const initiators = useMemo(() => {
    const names = [...(facets.data?.initiators ?? [])];
    if (filters.initiator && !names.includes(filters.initiator)) {
      names.push(filters.initiator);
    }
    return names.map((name) => option(name, name));
  }, [facets.data, filters.initiator]);
  const repositoryOptions = useMemo(() => {
    const names = (repositories.data ?? []).map((repository) => repository.name);
    if (filters.repository && !names.includes(filters.repository)) {
      names.push(filters.repository);
    }
    return names.sort((left, right) => left.localeCompare(right)).map((name) => option(name, name));
  }, [repositories.data, filters.repository]);

  const filtered = isFiltered(filters);
  const columns: Column<AuditEvent>[] = [
    {
      id: 'at',
      header: t('audit.table.time'),
      primary: true,
      cell: (event) => <DateTime value={event.at} />,
    },
    {
      id: 'domain',
      header: t('audit.table.domain'),
      cell: (event) => (
        <span title={event.domain} className={styles.domain}>
          {domainLabel(t, event.domain)}
        </span>
      ),
    },
    { id: 'type', header: t('audit.table.type'), cell: (event) => typeLabel(t, event.type) },
    {
      id: 'context',
      header: t('audit.table.context'),
      cell: (event) => <span className={styles.mono}>{event.context || '—'}</span>,
    },
    {
      id: 'initiator',
      header: t('audit.table.initiator'),
      cell: (event) => <span className={styles.mono}>{event.initiator || '—'}</span>,
    },
    { id: 'attributes', header: t('audit.table.attributes'), cell: (event) => <Attributes event={event} /> },
  ];
  const showFilters = !phone || open;
  const activeCount = Object.entries(filters).filter(([key, value]) => key !== 'q' && value !== '').length;

  return (
    <Page
      title={t('audit.page.title')}
      description={t('audit.page.description')}
      actions={
        <Button variant="soft" color="gray" onClick={() => void query.refetch()} loading={query.isRefetching}>
          <RotateCw size={16} aria-hidden="true" />
          {t('audit.page.refresh')}
        </Button>
      }
    >
      <section className={styles.filters} aria-label={t('audit.filters.label')}>
        <div className={styles.search}>
          <FieldShell id={searchId} label={t('audit.filters.search')}>
            <RadixTextField.Root
              id={searchId}
              type="search"
              value={text}
              placeholder={t('audit.filters.searchPlaceholder')}
              onChange={(event) => setText(event.target.value)}
            >
              <RadixTextField.Slot>
                <Search size={16} aria-hidden="true" />
              </RadixTextField.Slot>
            </RadixTextField.Root>
          </FieldShell>
        </div>
        {phone && (
          <div className={styles.toggle}>
            <Button variant="soft" color="gray" aria-expanded={open} onClick={() => setOpen(!open)}>
              <ListFilter size={16} aria-hidden="true" />
              {activeCount > 0
                ? t('audit.filters.toggleCount', { count: activeCount })
                : t('audit.filters.toggle')}
            </Button>
          </div>
        )}
        {showFilters && (
          <>
            <SelectField
              label={t('audit.filters.domain')}
              value={filters.domain || ALL}
              options={[option(ALL, t('audit.filters.allDomains')), ...domainOptions]}
              onChange={(value) => update({ domain: value === ALL ? '' : value })}
            />
            <SelectField
              label={t('audit.filters.type')}
              value={filters.type || ALL}
              options={[option(ALL, t('audit.filters.allTypes')), ...typeOptions]}
              onChange={(value) => update({ type: value === ALL ? '' : value })}
            />
            <SelectField
              label={t('audit.filters.initiator')}
              value={filters.initiator || ALL}
              options={[option(ALL, t('audit.filters.allInitiators')), ...initiators]}
              onChange={(value) => update({ initiator: value === ALL ? '' : value })}
            />
            <SelectField
              label={t('audit.filters.repository')}
              value={filters.repository || ALL}
              options={[option(ALL, t('audit.filters.allRepositories')), ...repositoryOptions]}
              onChange={(value) => update({ repository: value === ALL ? '' : value })}
            />
            <DateField
              label={t('audit.filters.since')}
              value={filters.since}
              onChange={(since) => update({ since })}
            />
            <DateField
              label={t('audit.filters.until')}
              value={filters.until}
              onChange={(until) => update({ until })}
            />
            <div className={styles.reset}>
              <Button
                variant="ghost"
                color="gray"
                disabled={!filtered}
                onClick={() => {
                  pushed.current = '';
                  setText('');
                  setParams(writeFilters(EMPTY_FILTERS), { replace: true });
                }}
              >
                {t('audit.filters.reset')}
              </Button>
            </div>
          </>
        )}
      </section>
      {events && events.length > 0 && (
        <Text size="2" color="gray" role="status">
          {query.hasNextPage
            ? t('audit.table.shownMore', { count: events.length })
            : t('audit.table.shown', { count: events.length })}
        </Text>
      )}
      <DataTable
        label={t('audit.table.label')}
        columns={columns}
        rows={events}
        getRowId={(event) => String(event.id)}
        loading={query.isPending}
        error={query.error}
        onRetry={() => void query.refetch()}
        paging="more"
        hasMore={query.hasNextPage}
        onLoadMore={() => void query.fetchNextPage()}
        loadingMore={query.isFetchingNextPage}
        empty={
          filtered ? (
            <EmptyState
              icon={SearchX}
              title={t('audit.table.noMatches')}
              description={t('audit.table.noMatchesText')}
            />
          ) : (
            <EmptyState
              icon={ScrollText}
              title={t('audit.table.emptyTitle')}
              description={t('audit.table.emptyText')}
            />
          )
        }
      />
    </Page>
  );
}
