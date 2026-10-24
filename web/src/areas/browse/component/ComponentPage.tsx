import { Badge, Button, Heading, IconButton, Select, Skeleton, TabNav, Text } from '@radix-ui/themes';
import { useQueries } from '@tanstack/react-query';
import { Copy, ExternalLink, PackageSearch } from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { Link, useLocation, useSearchParams } from 'react-router';
import { api, apiKey, path, useApi, withQuery } from '../../../api';
import { useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  Breadcrumbs,
  copyText,
  DateTime,
  EmptyState,
  ErrorState,
  formatName,
  useDocumentTitle,
  useToast,
} from '../../../ui';
import { componentKey, fromProxy, parseComponentKey } from '../links';
import {
  REPOSITORIES_PATH,
  type RepositoryItem,
  type SearchAsset,
  type SearchComponent,
  UI_PATH,
  type VersionEntry,
  type VersionsResponse,
} from '../types';
import styles from './ComponentPage.module.css';
import { componentGuideUrl, componentPath, componentSnippets, registryUrl } from './snippets';
import { type FileRow, FilesTab, RepositoriesTab, VersionsTab } from './tables';

const PREFIX = '/search/component/';
const TABS = ['versions', 'repos', 'files'] as const;
type Tab = 'overview' | (typeof TABS)[number];

export function locate(pathname: string): { key: string; tab: Tab } {
  const raw = pathname.startsWith(PREFIX) ? pathname.slice(PREFIX.length).replace(/\/+$/, '') : '';
  const slash = raw.lastIndexOf('/');
  const last = slash >= 0 ? raw.slice(slash + 1) : '';
  const tab = (TABS as readonly string[]).includes(last) ? (last as Tab) : 'overview';
  const encoded = tab === 'overview' ? raw : raw.slice(0, slash);
  try {
    return { key: decodeURIComponent(encoded), tab };
  } catch {
    return { key: encoded, tab };
  }
}

export function keyFromPath(pathname: string): string {
  return locate(pathname).key;
}

export function tabUrl(key: string, tab: Tab, version?: string): string {
  const base = `${PREFIX}${encodeURIComponent(key)}${tab === 'overview' ? '' : `/${tab}`}`;
  return version ? `${base}?version=${encodeURIComponent(version)}` : base;
}

export function mainAsset(format: string, assets: readonly SearchAsset[]): SearchAsset | undefined {
  const score = (asset: SearchAsset) => {
    if (format === 'docker' && asset.path.includes('/manifests/')) {
      return 1000;
    }
    if (format === 'maven2') {
      return asset.path.endsWith('.jar') ? 500 : asset.path.endsWith('.pom') ? 400 : 0;
    }
    return 0;
  };
  return [...assets].sort((left, right) => score(right) - score(left))[0];
}

function Dependencies({
  format,
  group,
  name,
  version,
  repository,
}: {
  format: string;
  group: string;
  name: string;
  version: string;
  repository: string;
}) {
  const t = useT();
  const toast = useToast();
  const { state } = useSession();
  const repositories = useApi<RepositoryItem[]>(REPOSITORIES_PATH);
  const found = repositories.data?.find((candidate) => candidate.name === repository);
  const snippets = componentSnippets({
    format,
    group,
    name,
    version,
    repository,
    repositoryType: found?.type,
    baseUrl: state.baseUrl,
    domain: found?.domain,
  });
  const [chosen, setChosen] = useState<string | undefined>(undefined);
  const current = snippets.find((snippet) => snippet.id === chosen) ?? snippets[0];
  if (!current) {
    return null;
  }
  const copy = async () => {
    try {
      await copyText(current.code);
      toast.success(t('ui.copy.copied'));
    } catch {
      toast.error(t('ui.copy.failed'));
    }
  };
  return (
    <section className={styles.card} aria-labelledby="component-dependencies">
      <Heading as="h2" size="5" weight="bold" id="component-dependencies" className={styles.cardTitle}>
        {t('browse.component.dependencies')}
      </Heading>
      <div className={styles.snippetBar}>
        <Select.Root value={current.id} onValueChange={setChosen}>
          <Select.Trigger className={styles.snippetSelect} aria-label={t('browse.component.chooseSnippet')} />
          <Select.Content position="popper">
            {snippets.map((snippet) => (
              <Select.Item key={snippet.id} value={snippet.id}>
                {snippet.label}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
        <IconButton
          variant="soft"
          aria-label={t('browse.component.copySnippet', { label: current.label })}
          onClick={copy}
        >
          <Copy size={16} aria-hidden="true" />
        </IconButton>
      </div>
      {current.hint && (
        <Text as="p" size="1" color="gray" className={styles.snippetHint}>
          {t(current.hint)}
        </Text>
      )}
      <pre className={styles.snippet}>
        <code>{current.code}</code>
      </pre>
    </section>
  );
}

function Facts({
  format,
  group,
  name,
  entry,
}: {
  format: string;
  group: string;
  name: string;
  entry: VersionEntry;
}) {
  const t = useT();
  const rows: Array<{ id: string; label: string; value: ReactNode; mono?: boolean }> = [
    {
      id: 'modified',
      label: t('browse.details.updated'),
      value: entry.lastModified ? <DateTime value={entry.lastModified} dateOnly /> : '—',
    },
    { id: 'format', label: t('browse.details.format'), value: formatName(format) },
  ];
  if (group) {
    rows.push({ id: 'group', label: t('browse.details.group'), value: group });
  }
  rows.push({ id: 'name', label: t('browse.details.name'), value: name });
  if (entry.version) {
    rows.push({ id: 'version', label: t('browse.details.version'), value: entry.version });
  }
  rows.push({
    id: 'repository',
    label: t('browse.details.repository'),
    value: entry.repositories.join(', '),
    mono: true,
  });
  return (
    <section className={styles.card} aria-labelledby="component-facts">
      <Heading as="h2" size="5" weight="bold" id="component-facts" className={styles.cardTitle}>
        {t('browse.component.facts')}
      </Heading>
      <dl className={styles.facts}>
        {rows.map((row) => (
          <div key={row.id} className={styles.fact}>
            <dt>{row.label}</dt>
            <dd data-mono={row.mono === true}>{row.value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function Overview({
  format,
  group,
  name,
  entry,
}: {
  format: string;
  group: string;
  name: string;
  entry: VersionEntry;
}) {
  const { state } = useSession();
  const repository = entry.repositories[0] ?? '';
  const hasSnippets =
    componentSnippets({ format, group, name, version: entry.version, repository, baseUrl: state.baseUrl })
      .length > 0;
  return (
    <div className={styles.overview} data-single={!hasSnippets}>
      {hasSnippets && (
        <Dependencies
          format={format}
          group={group}
          name={name}
          version={entry.version}
          repository={repository}
        />
      )}
      <Facts format={format} group={group} name={name} entry={entry} />
    </div>
  );
}

function Actions({
  format,
  group,
  name,
  version,
  proxied,
}: {
  format: string;
  group: string;
  name: string;
  version: string;
  proxied: boolean;
}) {
  const t = useT();
  const copied = componentPath(format, group, name, version);
  const registry = registryUrl(format, group, name);
  const guide = proxied ? componentGuideUrl(format, group, name, version) : null;
  if (!copied && !registry && !guide) {
    return null;
  }
  return (
    <div className={styles.actions}>
      {copied && <PathButton text={copied} />}
      {registry && (
        <Button asChild variant="soft" color="gray" highContrast>
          <a href={registry} target="_blank" rel="noreferrer">
            <ExternalLink size={14} aria-hidden="true" />
            {t('browse.component.registry')}
          </a>
        </Button>
      )}
      {guide && (
        <Button asChild variant="soft">
          <a href={guide} target="_blank" rel="noreferrer">
            <ExternalLink size={14} aria-hidden="true" />
            {t('browse.details.research')}
          </a>
        </Button>
      )}
    </div>
  );
}

function PathButton({ text }: { text: string }) {
  const t = useT();
  const toast = useToast();
  return (
    <Button
      variant="soft"
      aria-label={t('browse.component.copyPath')}
      onClick={async () => {
        try {
          await copyText(text);
          toast.success(t('ui.copy.copied'));
        } catch {
          toast.error(t('ui.copy.failed'));
        }
      }}
    >
      <Copy size={14} aria-hidden="true" />
      {t('browse.component.path')}
    </Button>
  );
}

function Frame({
  crumbs,
  title,
  children,
}: {
  crumbs: Array<{ label: string; to?: string }>;
  title: string;
  children: ReactNode;
}) {
  useDocumentTitle(title);
  return (
    <div className={styles.page}>
      <Breadcrumbs items={crumbs} />
      <Heading as="h1" size="6" weight="bold" className={styles.title}>
        {title}
      </Heading>
      {children}
    </div>
  );
}

export default function ComponentPage() {
  const t = useT();
  const location = useLocation();
  const [params] = useSearchParams();
  const { key, tab } = locate(location.pathname);
  const parsed = parseComponentKey(key);
  const versionsPath = parsed
    ? withQuery(`${UI_PATH}/search/versions`, {
        format: parsed.format,
        group: parsed.group,
        name: parsed.name,
      })
    : null;
  const versions = useApi<VersionsResponse>(versionsPath);
  const repositories = useApi<RepositoryItem[]>(REPOSITORIES_PATH);
  const list = versions.data?.versions ?? [];
  const asked = params.get('version');
  const entry = list.find((candidate) => candidate.version === asked) ?? list[0];
  const components = useQueries({
    queries: (entry?.componentIds ?? []).map((id) => {
      const target = path`/service/rest/v1/components/${id}`;
      return {
        queryKey: apiKey(target),
        queryFn: ({ signal }: { signal: AbortSignal }) => api.get<SearchComponent>(target, { signal }),
      };
    }),
  });
  const loaded = components.flatMap((result) => (result.data ? [result.data] : []));
  const files: FileRow[] = loaded.flatMap((component) =>
    component.assets.map((asset) => ({ ...asset, key: `${component.repository}|${asset.path}` })),
  );
  const format = versions.data?.format ?? parsed?.format ?? '';
  const group = parsed?.group ?? '';
  const name = parsed?.name ?? '';
  const search = { label: t('browse.component.search'), to: '/search' };

  if (!parsed) {
    const title = t('browse.component.invalidTitle');
    return (
      <Frame crumbs={[search, { label: title }]} title={title}>
        <EmptyState icon={PackageSearch} title={title} description={t('browse.component.invalidText')} />
      </Frame>
    );
  }
  if (versions.isPending) {
    return (
      <Frame crumbs={[search, { label: name }]} title={name}>
        <Skeleton height="200px" />
      </Frame>
    );
  }
  if (versions.isError) {
    return (
      <Frame crumbs={[search, { label: name }]} title={name}>
        <ErrorState error={versions.error} onRetry={() => versions.refetch()} />
      </Frame>
    );
  }
  if (!entry) {
    return (
      <Frame crumbs={[search, { label: name }]} title={t('browse.component.notFoundTitle')}>
        <EmptyState
          icon={PackageSearch}
          title={t('browse.component.notFoundTitle')}
          description={t('browse.component.notFoundText', { name })}
          action={
            <Button asChild variant="soft">
              <Link to="/search">{t('browse.component.toSearch')}</Link>
            </Button>
          }
        />
      </Frame>
    );
  }
  const own = componentKey(format, group, name);
  const order = list.map((item) => item.version);
  const compare = (left: string, right: string) => order.indexOf(right) - order.indexOf(left);
  const title = entry.version ? `${name} ${/^v\d/.test(entry.version) ? '' : 'v'}${entry.version}` : name;
  const crumbs = [
    search,
    { label: name, to: tabUrl(own, 'overview') },
    ...(entry.version ? [{ label: entry.version }] : []),
  ];
  const repositoryCount = new Set(list.flatMap((item) => item.repositories)).size;
  const loadingFiles = components.some((result) => result.isPending);
  const filesError = components.find((result) => result.isError)?.error;
  const tabs: Array<{ id: Tab; label: string; count?: number }> = [
    { id: 'overview', label: t('browse.component.overview') },
    { id: 'versions', label: t('browse.component.versionsTab'), count: list.length },
    { id: 'repos', label: t('browse.component.repositoriesTab'), count: repositoryCount },
    { id: 'files', label: t('browse.component.files'), count: loadingFiles ? undefined : files.length },
  ];
  return (
    <Frame crumbs={crumbs} title={title}>
      <Actions
        format={format}
        group={group}
        name={name}
        version={entry.version}
        proxied={fromProxy(repositories.data, entry.repositories)}
      />
      <TabNav.Root className={styles.tabs} aria-label={t('browse.component.tabs')}>
        {tabs.map((item) => (
          <TabNav.Link key={item.id} asChild active={tab === item.id}>
            <Link to={tabUrl(own, item.id, entry.version)}>
              {item.label}
              {item.count !== undefined && (
                <Badge size="1" color="gray" variant="soft" radius="full" className={styles.count}>
                  {item.count}
                </Badge>
              )}
            </Link>
          </TabNav.Link>
        ))}
      </TabNav.Root>
      {tab === 'overview' && <Overview format={format} group={group} name={name} entry={entry} />}
      {tab === 'versions' && (
        <VersionsTab
          versions={list}
          current={entry.version}
          hrefOf={(version) => tabUrl(own, 'versions', version)}
          compare={compare}
        />
      )}
      {tab === 'repos' && <RepositoriesTab versions={list} repositories={repositories.data ?? []} />}
      {tab === 'files' &&
        (filesError ? (
          <ErrorState error={filesError} />
        ) : loadingFiles ? (
          <Skeleton height="200px" />
        ) : (
          <FilesTab rows={files} />
        ))}
    </Frame>
  );
}
