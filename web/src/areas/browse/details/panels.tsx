import { Button, Heading, Skeleton, Text } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { ChevronDown, Download, ExternalLink } from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { api, apiKey, path, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useSession } from '../../../auth/state';
import { formatNumber, useI18n, useT } from '../../../i18n';
import { CopyButton, DateTime, ErrorState, FileSize, formatName, useToast } from '../../../ui';
import { componentTitle } from '../links';
import { parentOf } from '../tree/model';
import {
  type AssetDetails,
  type FolderDeleted,
  formatAttributes,
  type RepositoryItem,
  type TreeListing,
  type TreeNode,
  UI_PATH,
} from '../types';
import { supportsUsage, type UsageContext, UsageSnippets } from '../usage';
import { DeleteButton } from './DeleteButton';
import styles from './Details.module.css';

export const SEARCH_READ = 'astrum:search:read';

export interface PanelProps {
  repository: RepositoryItem;
  onNavigate(path: string): void;
}

export function treePath(repository: string, folder: string): string {
  return withQuery(`${UI_PATH}${path`/browse/${repository}`}`, { path: folder });
}

export function assetPath(repository: string, file: string): string {
  return withQuery(`${UI_PATH}${path`/browse/${repository}/asset`}`, { path: file });
}

const GUIDE = 'https://guide.globalart.dev/component';
const GUIDE_FORMATS: Record<string, string> = {
  maven2: 'maven',
  npm: 'npm',
  pypi: 'pypi',
  docker: 'docker',
  helm: 'helm',
  nuget: 'nuget',
};

export function guideUrl(
  format: string,
  component: { group: string; name: string; version: string },
  referrer: 'browse-tree-component' | 'repo-browse-asset',
): string | null {
  const id = GUIDE_FORMATS[format];
  if (!id || component.version === '') {
    return null;
  }
  const name = format === 'maven2' ? `${component.group}:${component.name}` : component.name;
  return `${GUIDE}/${id}/${encodeURIComponent(name)}/${encodeURIComponent(component.version)}?referrer=${referrer}`;
}

export function attributeTitle(format: string): string {
  return format.charAt(0).toUpperCase() + format.slice(1);
}

function useContentChanged(): () => Promise<void> {
  const invalidate = useInvalidateApi();
  return async () => {
    await Promise.all([invalidate(`${UI_PATH}/`), invalidate('/service/rest/v1/')]);
  };
}

export function Card({ children, label }: { children: ReactNode; label?: string }) {
  return (
    <section className={styles.card} aria-label={label}>
      {children}
    </section>
  );
}

function PanelHeader({
  path: shown,
  title,
  actions,
}: {
  path: string;
  title: ReactNode;
  actions?: ReactNode;
}) {
  const t = useT();
  return (
    <header className={styles.header}>
      <div className={styles.heading}>
        <span className={styles.path}>
          <code>{shown}</code>
          <CopyButton text={shown} label={t('browse.details.copyPath')} />
        </span>
        <Heading as="h2" size="6" weight="bold" className={styles.title}>
          {title}
        </Heading>
      </div>
      {actions && <div className={styles.actions}>{actions}</div>}
    </header>
  );
}

export function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <Text as="p" size="3" weight="bold" className={styles.sectionTitle}>
      {children}
    </Text>
  );
}

export interface Field {
  id: string;
  label: string;
  value: ReactNode;
  mono?: boolean;
}

export function Fields({ items, label }: { items: Field[]; label?: string }) {
  return (
    <dl className={styles.fields} aria-label={label}>
      {items.map((item) => (
        <div key={item.id} className={styles.field}>
          <dt>{item.label}</dt>
          <dd data-mono={item.mono === true}>{item.value}</dd>
        </div>
      ))}
    </dl>
  );
}

function DetailsCard({ items }: { items: Field[] }) {
  const t = useT();
  return (
    <Card label={t('browse.details.details')}>
      <SectionTitle>{t('browse.details.details')}</SectionTitle>
      <Fields items={items} />
    </Card>
  );
}

function Usage({ context }: { context: UsageContext }) {
  const t = useT();
  if (!supportsUsage(context.format)) {
    return null;
  }
  return (
    <div className={styles.usage}>
      <SectionTitle>{t('browse.details.usage')}</SectionTitle>
      <UsageSnippets context={context} />
    </div>
  );
}

function GuideButton({ href }: { href: string | null }) {
  const t = useT();
  if (!href) {
    return null;
  }
  return (
    <Button asChild variant="soft" color="gray" highContrast>
      <a href={href} target="_blank" rel="noreferrer">
        <ExternalLink size={16} aria-hidden="true" />
        {t('browse.details.research')}
      </a>
    </Button>
  );
}

function VersionTitle({
  name,
  version,
  separator = ':',
}: {
  name: string;
  version: string;
  separator?: string;
}) {
  return (
    <>
      {name}
      {version && (
        <span className={styles.version}>
          {separator}
          {version}
        </span>
      )}
    </>
  );
}

function shownPath(value: string): string {
  return value.length > 1 && value.endsWith('/') ? value.slice(0, -1) : value;
}

function depthOf(value: string): number {
  return value.split('/').filter(Boolean).length;
}

export function FolderPanel({ repository, node, onNavigate }: PanelProps & { node: TreeNode }) {
  const t = useT();
  const { language } = useI18n();
  const toast = useToast();
  const changed = useContentChanged();
  const listing = useApi<TreeListing>(treePath(repository.name, node.path));
  const deletable = repository.permissions.delete && repository.type !== 'group';
  const children = listing.data?.children ?? [];
  const folders = children.filter((child) => child.kind !== 'asset').length;
  const count = (value: number) => (listing.data ? formatNumber(language, value) : '—');
  const remove = async () => {
    const result = await api.delete<FolderDeleted>(treePath(repository.name, node.path));
    onNavigate(parentOf(node.path));
    await changed();
    toast.success(
      t('browse.details.folderDeleted', { name: node.name }),
      [
        t('browse.details.deletedAssets', { count: result.deletedAssets }),
        t('browse.details.deletedComponents', { count: result.deletedComponents }),
        result.skippedAssets > 0 ? t('browse.details.skippedAssets', { count: result.skippedAssets }) : '',
      ]
        .filter(Boolean)
        .join(' · '),
    );
  };
  return (
    <div className={styles.panel}>
      <Card>
        <PanelHeader
          path={shownPath(node.path)}
          title={node.name}
          actions={
            deletable && (
              <DeleteButton
                label={t('browse.details.deleteFolder')}
                title={t('browse.details.deleteFolderTitle')}
                description={t('browse.details.deleteFolderText', { name: node.name })}
                repository={repository.name}
                onConfirm={remove}
              />
            )
          }
        />
        {deletable && (
          <Text as="p" size="2" color="gray" className={styles.note}>
            {t('browse.details.folderWarning')}
          </Text>
        )}
      </Card>
      <DetailsCard
        items={[
          { id: 'repository', label: t('browse.details.repository'), value: repository.name },
          { id: 'path', label: t('browse.details.path'), value: shownPath(node.path), mono: true },
          {
            id: 'depth',
            label: t('browse.details.depth'),
            value: formatNumber(language, depthOf(node.path)),
          },
          { id: 'children', label: t('browse.details.children'), value: count(children.length) },
          { id: 'folders', label: t('browse.details.folders'), value: count(folders) },
          { id: 'files', label: t('browse.details.fileCount'), value: count(children.length - folders) },
        ]}
      />
    </div>
  );
}

export function ComponentPanel({ repository, node, onNavigate }: PanelProps & { node: TreeNode }) {
  const t = useT();
  const { language } = useI18n();
  const toast = useToast();
  const changed = useContentChanged();
  const { state } = useSession();
  const listing = useApi<TreeListing>(treePath(repository.name, node.path));
  const component = node.component ?? { group: '', name: node.name, version: '' };
  const deletable = repository.permissions.delete && repository.type !== 'group' && node.componentId;
  const title = componentTitle(repository.format, component.group, component.name);
  const downloads = (listing.data?.children ?? []).reduce(
    (sum, child) => sum + (child.downloadCount ?? 0),
    0,
  );
  const remove = async () => {
    await api.delete(path`/service/rest/v1/components/${node.componentId ?? ''}`);
    onNavigate(parentOf(node.path));
    await changed();
    toast.success(t('browse.details.componentDeleted', { name: `${title}:${component.version}` }));
  };
  return (
    <div className={styles.panel}>
      <Card>
        <PanelHeader
          path={shownPath(node.path)}
          title={<VersionTitle name={component.name} version={component.version} />}
          actions={
            <>
              <GuideButton href={guideUrl(repository.format, component, 'browse-tree-component')} />
              {deletable && (
                <DeleteButton
                  label={t('browse.details.deleteComponent')}
                  title={t('browse.details.deleteComponentTitle')}
                  description={t('browse.details.deleteComponentText', {
                    name: `${title}:${component.version}`,
                  })}
                  repository={repository.name}
                  onConfirm={remove}
                />
              )}
            </>
          }
        />
        <Usage
          context={{
            format: repository.format,
            repositoryName: repository.name,
            repositoryType: repository.type,
            baseUrl: state.baseUrl,
            component,
          }}
        />
      </Card>
      <DetailsCard
        items={[
          {
            id: 'repository',
            label: t('browse.details.repository'),
            value: node.repository ?? repository.name,
          },
          { id: 'format', label: t('browse.details.format'), value: formatName(repository.format) },
          ...(component.group
            ? [{ id: 'group', label: t('browse.details.group'), value: component.group, mono: true }]
            : []),
          { id: 'name', label: t('browse.details.name'), value: component.name },
          { id: 'version', label: t('browse.details.version'), value: component.version, mono: true },
          {
            id: 'downloads',
            label: t('browse.details.downloads'),
            value: listing.data ? formatNumber(language, downloads) : '—',
          },
        ]}
      />
    </div>
  );
}

function checksumFields(details: AssetDetails): Field[] {
  const { md5, sha1, sha256, sha512 } = details.checksum;
  const entries: Array<[string, string, string | null | undefined]> =
    details.format === 'docker'
      ? [
          ['digest', 'Digest', sha256 ? `sha256:${sha256}` : null],
          ['sha1', 'SHA-1', sha1],
        ]
      : [
          ['md5', 'MD5', md5],
          ['sha1', 'SHA-1', sha1],
          ['sha256', 'SHA-256', sha256],
          ['sha512', 'SHA-512', sha512],
        ];
  return entries
    .filter((entry): entry is [string, string, string] => typeof entry[2] === 'string' && entry[2] !== '')
    .map(([id, label, value]) => ({ id, label, value, mono: true }));
}

function plain(value: unknown): string {
  if (value === null || value === undefined) {
    return '';
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function AttributesCard({ format, attributes }: { format: string; attributes: Record<string, unknown> }) {
  const t = useT();
  const [open, setOpen] = useState(true);
  const entries = Object.entries(attributes).filter(([, value]) => plain(value) !== '');
  if (entries.length === 0) {
    return null;
  }
  const title = attributeTitle(format);
  return (
    <Card label={t('browse.details.attributes')}>
      <SectionTitle>{t('browse.details.attributes')}</SectionTitle>
      <div className={styles.accordion}>
        <button
          type="button"
          className={styles.accordionHead}
          aria-expanded={open}
          onClick={() => setOpen(!open)}
        >
          {title}
          <ChevronDown size={16} aria-hidden="true" className={styles.accordionIcon} data-open={open} />
        </button>
        {open && (
          <Fields
            label={title}
            items={entries.map(([key, value]) => ({ id: key, label: key, value: plain(value) }))}
          />
        )}
      </div>
    </Card>
  );
}

function GeneratedPanel({ repository, node }: { repository: RepositoryItem; node: TreeNode }) {
  const t = useT();
  return (
    <div className={styles.panel}>
      <Card>
        <PanelHeader
          path={node.path}
          title={node.name}
          actions={
            node.downloadUrl && (
              <Button asChild variant="soft" color="gray">
                <a href={node.downloadUrl} download={node.name}>
                  <Download size={16} aria-hidden="true" />
                  {t('browse.details.download')}
                </a>
              </Button>
            )
          }
        />
      </Card>
      <DetailsCard
        items={[
          { id: 'repository', label: t('browse.details.repository'), value: repository.name },
          { id: 'format', label: t('browse.details.format'), value: formatName(repository.format) },
        ]}
      />
    </div>
  );
}

function assetTitle(details: AssetDetails, fileName: string): ReactNode {
  const component = details.component;
  if (!component || details.format === 'raw' || component.version === '') {
    return fileName;
  }
  const name =
    details.format === 'docker' || details.format === 'npm'
      ? componentTitle(details.format, component.group, component.name)
      : component.name;
  return (
    <VersionTitle name={name} version={component.version} separator={details.format === 'npm' ? '@' : ':'} />
  );
}

export function AssetPanel({ repository, node, onNavigate }: PanelProps & { node: TreeNode }) {
  if (node.generated) {
    return <GeneratedPanel repository={repository} node={node} />;
  }
  return <StoredAssetPanel repository={repository} node={node} onNavigate={onNavigate} />;
}

function StoredAssetPanel({ repository, node, onNavigate }: PanelProps & { node: TreeNode }) {
  const t = useT();
  const { language } = useI18n();
  const toast = useToast();
  const changed = useContentChanged();
  const client = useQueryClient();
  const { state } = useSession();
  const key = assetPath(repository.name, node.assetPath ?? node.path);
  const query = useApi<AssetDetails>(key);
  if (query.isPending) {
    return (
      <div className={styles.panel} aria-busy="true">
        <Skeleton height="120px" />
        <Skeleton height="320px" />
      </div>
    );
  }
  if (query.isError) {
    return <ErrorState error={query.error} onRetry={() => query.refetch()} />;
  }
  const details = query.data;
  const component = details.component;
  const attributes = formatAttributes(details.format, details);
  const remove = async () => {
    await api.delete(path`/service/rest/v1/assets/${details.id}`);
    onNavigate(parentOf(node.path));
    client.removeQueries({ queryKey: apiKey(key) });
    await changed();
    toast.success(t('browse.details.assetDeleted', { name: node.name }));
  };
  const items: Field[] = [
    { id: 'repository', label: t('browse.details.repository'), value: details.repository },
    { id: 'format', label: t('browse.details.format'), value: formatName(details.format) },
  ];
  if (component) {
    if (component.group) {
      items.push({ id: 'group', label: t('browse.details.group'), value: component.group, mono: true });
    }
    items.push({ id: 'component', label: t('browse.details.component'), value: component.name });
    if (component.version) {
      items.push({ id: 'version', label: t('browse.details.version'), value: component.version, mono: true });
    }
  }
  items.push(
    { id: 'uploader', label: t('browse.details.uploader'), value: details.uploader },
    { id: 'contentType', label: t('browse.details.contentType'), value: details.contentType, mono: true },
    { id: 'size', label: t('browse.details.size'), value: <FileSize bytes={details.fileSize} /> },
    { id: 'created', label: t('browse.details.created'), value: <DateTime value={details.blobCreated} /> },
    {
      id: 'lastDownloaded',
      label: t('browse.details.lastDownloaded'),
      value: details.lastDownloaded ? <DateTime value={details.lastDownloaded} /> : t('browse.details.never'),
    },
    {
      id: 'downloads',
      label: t('browse.details.downloads'),
      value: formatNumber(language, details.downloadCount),
    },
    ...checksumFields(details),
  );
  if (details.blobRef) {
    items.push({ id: 'blobRef', label: t('browse.details.blobRef'), value: details.blobRef, mono: true });
  }
  const fileName = details.path.split('/').pop() ?? node.name;
  return (
    <div className={styles.panel}>
      <Card>
        <PanelHeader
          path={details.path}
          title={assetTitle(details, fileName)}
          actions={
            <>
              {component && <GuideButton href={guideUrl(details.format, component, 'repo-browse-asset')} />}
              <Button asChild variant="soft" color="gray">
                <a href={details.downloadUrl} download={fileName}>
                  <Download size={16} aria-hidden="true" />
                  {t('browse.details.download')}
                </a>
              </Button>
              {details.canDelete && (
                <DeleteButton
                  label={t('browse.details.deleteAsset')}
                  title={t('browse.details.deleteAssetTitle')}
                  description={t('browse.details.deleteAssetText', { name: fileName })}
                  repository={details.repository}
                  onConfirm={remove}
                />
              )}
            </>
          }
        />
        <Usage
          context={{
            format: details.format,
            repositoryName: repository.name,
            repositoryType: repository.type,
            baseUrl: state.baseUrl,
            component: component && details.format !== 'raw' ? component : undefined,
            path: details.path,
            attributes,
          }}
        />
      </Card>
      <DetailsCard items={items} />
      <AttributesCard format={details.format} attributes={attributes} />
    </div>
  );
}
