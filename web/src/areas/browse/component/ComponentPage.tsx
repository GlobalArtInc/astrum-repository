import {
  Button,
  Flex,
  IconButton,
  Link as RadixLink,
  Select,
  Skeleton,
  Text,
  Tooltip,
} from '@radix-ui/themes';
import { useQueries } from '@tanstack/react-query';
import { Download, PackageSearch } from 'lucide-react';
import { useState } from 'react';
import { Link, useLocation, useSearchParams } from 'react-router';
import { api, apiKey, path, useApi, withQuery } from '../../../api';
import { useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  type Column,
  DataTable,
  DateTime,
  EmptyState,
  ErrorState,
  FileSize,
  FormatBadge,
  formatName,
  Page,
  QueryTabs,
} from '../../../ui';
import { componentPageUrl, componentTitle, parseComponentKey } from '../links';
import { AttributeList, DetailList, Section } from '../parts';
import { browseUrl } from '../tree/model';
import {
  formatAttributes,
  REPOSITORIES_PATH,
  type RepositoryItem,
  type SearchAsset,
  type SearchComponent,
  UI_PATH,
  type VersionEntry,
  type VersionsResponse,
} from '../types';
import { supportsUsage, UsageSnippets } from '../usage';
import styles from './ComponentPage.module.css';

const PREFIX = '/search/component/';

export function keyFromPath(pathname: string): string {
  const raw = pathname.startsWith(PREFIX) ? pathname.slice(PREFIX.length) : '';
  try {
    return decodeURIComponent(raw.replace(/\/+$/, ''));
  } catch {
    return raw;
  }
}

export function mainAsset(format: string, assets: readonly SearchAsset[]): SearchAsset | undefined {
  const score = (asset: SearchAsset) => {
    const attributes = formatAttributes(format, asset);
    const size = Object.keys(attributes).length;
    if (format === 'docker' && asset.path.includes('/manifests/')) {
      return 1000 + size;
    }
    if (format === 'maven2') {
      const extension = attributes.extension;
      return (extension === 'jar' ? 500 : extension === 'pom' ? 400 : 0) + size;
    }
    return size;
  };
  return [...assets].sort((left, right) => score(right) - score(left))[0];
}

interface FileRow extends SearchAsset {
  key: string;
}

function VersionPicker({
  versions,
  current,
  onChange,
}: {
  versions: VersionEntry[];
  current: string;
  onChange(version: string): void;
}) {
  const t = useT();
  return (
    <Select.Root value={current} onValueChange={onChange}>
      <label htmlFor="component-version" className="visually-hidden">
        {t('browse.component.versionPicker')}
      </label>
      <Select.Trigger id="component-version" className={styles.picker} />
      <Select.Content position="popper">
        {versions.map((entry) => (
          <Select.Item key={entry.version} value={entry.version}>
            {entry.version}
          </Select.Item>
        ))}
      </Select.Content>
    </Select.Root>
  );
}

function Overview({
  format,
  group,
  name,
  entry,
  main,
}: {
  format: string;
  group: string;
  name: string;
  entry: VersionEntry;
  main: SearchAsset | undefined;
}) {
  const t = useT();
  const attributes = main ? formatAttributes(format, main) : {};
  return (
    <Flex direction="column" gap="5">
      <Section title={t('browse.component.summary')}>
        <DetailList
          items={[
            { id: 'format', label: t('browse.details.format'), value: <FormatBadge format={format} /> },
            ...(group ? [{ id: 'group', label: t('browse.details.group'), value: group, mono: true }] : []),
            { id: 'name', label: t('browse.details.name'), value: name, mono: true },
            { id: 'version', label: t('browse.details.version'), value: entry.version, mono: true },
            {
              id: 'repositories',
              label: t('browse.component.repositories'),
              value: (
                <span className={styles.links}>
                  {entry.repositories.map((repository) => (
                    <RadixLink asChild key={repository}>
                      <Link to={browseUrl(repository)}>{repository}</Link>
                    </RadixLink>
                  ))}
                </span>
              ),
            },
            {
              id: 'modified',
              label: t('browse.component.lastModified'),
              value: entry.lastModified ? <DateTime value={entry.lastModified} /> : '—',
            },
          ]}
        />
      </Section>
      {Object.keys(attributes).length > 0 && (
        <Section title={t('browse.details.attributes', { format: formatName(format) })}>
          <AttributeList
            attributes={attributes}
            label={t('browse.details.attributes', { format: formatName(format) })}
          />
        </Section>
      )}
    </Flex>
  );
}

function Files({ rows, loading, error }: { rows: FileRow[]; loading: boolean; error: unknown }) {
  const t = useT();
  const columns: Column<FileRow>[] = [
    {
      id: 'path',
      header: t('browse.component.file'),
      primary: true,
      cell: (row) => <span className={styles.mono}>{row.path}</span>,
      sortValue: (row) => row.path,
    },
    {
      id: 'repository',
      header: t('browse.details.repository'),
      cell: (row) => row.repository,
      sortValue: (row) => row.repository,
    },
    {
      id: 'size',
      header: t('browse.details.size'),
      align: 'right',
      cell: (row) => <FileSize bytes={row.fileSize} />,
      sortValue: (row) => row.fileSize,
    },
    {
      id: 'modified',
      header: t('browse.details.updated'),
      cell: (row) => <DateTime value={row.lastModified} />,
      sortValue: (row) => new Date(row.lastModified),
      hideOnPhone: true,
    },
    {
      id: 'download',
      header: t('browse.details.download'),
      align: 'right',
      cell: (row) => (
        <Tooltip content={t('browse.component.downloadFile', { name: row.path })}>
          <IconButton asChild variant="ghost" size="1">
            <a
              href={row.downloadUrl}
              download
              aria-label={t('browse.component.downloadFile', { name: row.path })}
            >
              <Download size={15} aria-hidden="true" />
            </a>
          </IconButton>
        </Tooltip>
      ),
    },
  ];
  return (
    <DataTable
      label={t('browse.component.files')}
      columns={columns}
      rows={rows}
      loading={loading}
      error={error}
      getRowId={(row) => row.key}
      rowHref={(row) => browseUrl(row.repository, row.path)}
      initialSort={{ column: 'path', direction: 'asc' }}
    />
  );
}

function Usage({
  format,
  group,
  name,
  entry,
  components,
  repositories,
}: {
  format: string;
  group: string;
  name: string;
  entry: VersionEntry;
  components: SearchComponent[];
  repositories: RepositoryItem[] | undefined;
}) {
  const t = useT();
  const { state } = useSession();
  const [chosen, setChosen] = useState<string | undefined>(undefined);
  const repository = chosen && entry.repositories.includes(chosen) ? chosen : entry.repositories[0];
  if (!supportsUsage(format)) {
    return (
      <Text as="p" size="2" color="gray">
        {t('browse.usage.none', { format: formatName(format) })}
      </Text>
    );
  }
  if (!repository) {
    return null;
  }
  const item = repositories?.find((candidate) => candidate.name === repository);
  const component = components.find((candidate) => candidate.repository === repository);
  const main = component ? mainAsset(format, component.assets) : undefined;
  return (
    <Flex direction="column" gap="4">
      {entry.repositories.length > 1 && (
        <Flex direction="column" gap="1" className={styles.repositoryPicker}>
          <Text as="label" size="2" weight="medium" htmlFor="usage-repository">
            {t('browse.component.usageRepository')}
          </Text>
          <Select.Root value={repository} onValueChange={setChosen}>
            <Select.Trigger id="usage-repository" />
            <Select.Content position="popper">
              {entry.repositories.map((candidate) => (
                <Select.Item key={candidate} value={candidate}>
                  {candidate}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </Flex>
      )}
      <UsageSnippets
        key={repository}
        context={{
          format,
          repositoryName: repository,
          repositoryType: item?.type ?? 'hosted',
          baseUrl: state.baseUrl,
          component: { group, name, version: entry.version },
          path: format === 'raw' || format === 'docker' ? main?.path : undefined,
          attributes: main ? formatAttributes(format, main) : undefined,
        }}
      />
    </Flex>
  );
}

function Versions({
  format,
  group,
  name,
  versions,
  current,
}: {
  format: string;
  group: string;
  name: string;
  versions: VersionEntry[];
  current: string;
}) {
  const t = useT();
  const columns: Column<VersionEntry>[] = [
    {
      id: 'version',
      header: t('browse.details.version'),
      primary: true,
      cell: (row) => (
        <span className={styles.mono}>
          {row.version}
          {row.version === current && (
            <Text size="1" color="gray">
              {' '}
              {t('browse.component.shown')}
            </Text>
          )}
        </span>
      ),
    },
    {
      id: 'repositories',
      header: t('browse.component.repositories'),
      cell: (row) => row.repositories.join(', '),
    },
    {
      id: 'modified',
      header: t('browse.component.lastModified'),
      cell: (row) => (row.lastModified ? <DateTime value={row.lastModified} /> : '—'),
    },
  ];
  return (
    <DataTable
      label={t('browse.component.versions')}
      columns={columns}
      rows={versions}
      getRowId={(row) => row.version}
      rowHref={(row) => componentPageUrl(format, group, name, row.version)}
      pageSize={50}
      paging="more"
    />
  );
}

export default function ComponentPage() {
  const t = useT();
  const location = useLocation();
  const [params, setParams] = useSearchParams();
  const key = keyFromPath(location.pathname);
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
  const title = parsed ? componentTitle(format, group, name) : t('browse.component.invalidTitle');
  const crumbs = [{ label: t('browse.component.search'), to: '/search' }, { label: title }];

  if (!parsed) {
    return (
      <Page title={title} breadcrumbs={crumbs}>
        <EmptyState icon={PackageSearch} title={title} description={t('browse.component.invalidText')} />
      </Page>
    );
  }
  if (versions.isPending) {
    return (
      <Page title={title} breadcrumbs={crumbs}>
        <Skeleton height="200px" />
      </Page>
    );
  }
  if (versions.isError) {
    return (
      <Page title={title} breadcrumbs={crumbs}>
        <ErrorState error={versions.error} onRetry={() => versions.refetch()} />
      </Page>
    );
  }
  if (!entry) {
    return (
      <Page title={title} breadcrumbs={crumbs}>
        <EmptyState
          icon={PackageSearch}
          title={t('browse.component.notFoundTitle')}
          description={t('browse.component.notFoundText', { name: title })}
          action={
            <Button asChild variant="soft">
              <Link to="/search">{t('browse.component.toSearch')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }
  const loadingFiles = components.some((result) => result.isPending);
  const filesError = components.find((result) => result.isError)?.error;
  const main = mainAsset(format, loaded[0]?.assets ?? []);
  const chooseVersion = (version: string) => {
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        next.set('version', version);
        return next;
      },
      { replace: false },
    );
  };
  return (
    <Page
      title={title}
      documentTitle={`${title} ${entry.version}`}
      breadcrumbs={crumbs}
      description={
        <Flex as="span" gap="2" align="center" wrap="wrap">
          <FormatBadge format={format} />
          <span>{t('browse.component.versionCount', { count: list.length })}</span>
        </Flex>
      }
      actions={<VersionPicker versions={list} current={entry.version} onChange={chooseVersion} />}
    >
      <QueryTabs
        tabs={[
          {
            value: 'overview',
            label: t('browse.component.overview'),
            content: <Overview format={format} group={group} name={name} entry={entry} main={main} />,
          },
          {
            value: 'files',
            label: t('browse.component.filesTab', { count: files.length }),
            content: <Files rows={files} loading={loadingFiles} error={filesError} />,
          },
          {
            value: 'usage',
            label: t('browse.component.usage'),
            content: (
              <Usage
                format={format}
                group={group}
                name={name}
                entry={entry}
                components={loaded}
                repositories={repositories.data}
              />
            ),
          },
          {
            value: 'versions',
            label: t('browse.component.versionsTab', { count: list.length }),
            content: (
              <Versions format={format} group={group} name={name} versions={list} current={entry.version} />
            ),
          },
        ]}
      />
    </Page>
  );
}
