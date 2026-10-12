import { Button, Flex, Heading, Skeleton, Text } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { Download, ExternalLink, File, Folder, Link2, Package } from 'lucide-react';
import type { ReactNode } from 'react';
import { Link } from 'react-router';
import { api, apiKey, path, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useCan, useSession } from '../../../auth/state';
import { formatNumber, useI18n, useT } from '../../../i18n';
import {
  CopyButton,
  copyText,
  DateTime,
  ErrorState,
  FileSize,
  FormatBadge,
  formatName,
  useToast,
} from '../../../ui';
import { componentPageUrl, componentTitle } from '../links';
import { AttributeList, type DetailItem, DetailList, Section } from '../parts';
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
import { supportsUsage, UsageSnippets } from '../usage';
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

function useContentChanged(): () => Promise<void> {
  const invalidate = useInvalidateApi();
  return async () => {
    await Promise.all([invalidate(`${UI_PATH}/`), invalidate('/service/rest/v1/')]);
  };
}

function PanelHeader({
  icon,
  path: shown,
  title,
  actions,
}: {
  icon: ReactNode;
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
        <Heading as="h2" size="5" className={styles.title}>
          <span className={styles.titleIcon} aria-hidden="true">
            {icon}
          </span>
          <span className={styles.titleText}>{title}</span>
        </Heading>
      </div>
      {actions && <div className={styles.actions}>{actions}</div>}
    </header>
  );
}

function ComponentLink({
  format,
  group,
  name,
  version,
}: {
  format: string;
  group: string;
  name: string;
  version: string;
}) {
  const t = useT();
  const can = useCan(SEARCH_READ);
  if (!can) {
    return null;
  }
  return (
    <Button asChild variant="soft">
      <Link to={componentPageUrl(format, group, name, version)}>
        <ExternalLink size={15} aria-hidden="true" />
        {t('browse.details.componentPage')}
      </Link>
    </Button>
  );
}

function VersionTitle({ name, version }: { name: string; version: string }) {
  return (
    <>
      {name}
      {version && <span className={styles.version}>:{version}</span>}
    </>
  );
}

export function FolderPanel({ repository, node, onNavigate }: PanelProps & { node: TreeNode }) {
  const t = useT();
  const { language } = useI18n();
  const toast = useToast();
  const changed = useContentChanged();
  const deletable = repository.permissions.delete && repository.type !== 'group';
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
      <PanelHeader
        icon={<Folder size={20} />}
        path={node.path}
        title={node.name}
        actions={
          deletable && (
            <DeleteButton
              label={t('browse.details.deleteFolder')}
              title={t('browse.details.deleteFolderTitle', { name: node.name })}
              description={t('browse.details.deleteFolderText', {
                path: node.path,
                count: node.childCount ?? 0,
                repository: repository.name,
              })}
              confirmLabel={t('browse.details.delete')}
              onConfirm={remove}
            />
          )
        }
      />
      <Section title={t('browse.details.details')}>
        <DetailList
          items={[
            { id: 'repository', label: t('browse.details.repository'), value: repository.name },
            { id: 'path', label: t('browse.details.path'), value: node.path, mono: true },
            {
              id: 'children',
              label: t('browse.details.children'),
              value: formatNumber(language, node.childCount ?? 0),
            },
          ]}
        />
      </Section>
    </div>
  );
}

function FileList({ listing, onNavigate }: { listing: TreeListing; onNavigate(path: string): void }) {
  const t = useT();
  return (
    <ul className={styles.files} aria-label={t('browse.details.files')}>
      {listing.children.map((child) => (
        <li key={child.path} className={styles.file}>
          <button type="button" className={styles.fileName} onClick={() => onNavigate(child.path)}>
            {child.kind === 'asset' ? (
              <File size={14} aria-hidden="true" />
            ) : (
              <Folder size={14} aria-hidden="true" />
            )}
            <span>{child.name}</span>
          </button>
          {child.size !== undefined && (
            <Text size="1" color="gray" className={styles.fileSize}>
              <FileSize bytes={child.size} />
            </Text>
          )}
        </li>
      ))}
    </ul>
  );
}

export function ComponentPanel({ repository, node, onNavigate }: PanelProps & { node: TreeNode }) {
  const t = useT();
  const toast = useToast();
  const changed = useContentChanged();
  const { state } = useSession();
  const listing = useApi<TreeListing>(treePath(repository.name, node.path));
  const component = node.component ?? { group: '', name: node.name, version: '' };
  const deletable = repository.permissions.delete && repository.type !== 'group' && node.componentId;
  const title = componentTitle(repository.format, component.group, component.name);
  const remove = async () => {
    await api.delete(path`/service/rest/v1/components/${node.componentId ?? ''}`);
    onNavigate(parentOf(node.path));
    await changed();
    toast.success(t('browse.details.componentDeleted', { name: `${title}:${component.version}` }));
  };
  return (
    <div className={styles.panel}>
      <PanelHeader
        icon={<Package size={20} />}
        path={node.path}
        title={<VersionTitle name={component.name} version={component.version} />}
        actions={
          <>
            <ComponentLink format={repository.format} {...component} />
            {deletable && (
              <DeleteButton
                label={t('browse.details.deleteComponent')}
                title={t('browse.details.deleteComponentTitle')}
                description={t('browse.details.deleteComponentText', {
                  name: `${title}:${component.version}`,
                  repository: repository.name,
                })}
                confirmLabel={t('browse.details.delete')}
                onConfirm={remove}
              />
            )}
          </>
        }
      />
      {supportsUsage(repository.format) && (
        <Section title={t('browse.details.usage')}>
          <UsageSnippets
            context={{
              format: repository.format,
              repositoryName: repository.name,
              repositoryType: repository.type,
              baseUrl: state.baseUrl,
              component,
            }}
          />
        </Section>
      )}
      <Section title={t('browse.details.details')}>
        <DetailList
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
            { id: 'name', label: t('browse.details.name'), value: component.name, mono: true },
            { id: 'version', label: t('browse.details.version'), value: component.version, mono: true },
          ]}
        />
      </Section>
      <Section title={t('browse.details.files')}>
        {listing.isPending ? (
          <Skeleton height="80px" />
        ) : listing.isError ? (
          <ErrorState error={listing.error} onRetry={() => listing.refetch()} />
        ) : (
          <FileList listing={listing.data} onNavigate={onNavigate} />
        )}
      </Section>
    </div>
  );
}

function checksumItems(details: AssetDetails, t: ReturnType<typeof useT>): DetailItem[] {
  const entries: Array<[string, string, string | null | undefined]> = [
    ['sha1', 'SHA-1', details.checksum.sha1],
    ['sha256', 'SHA-256', details.checksum.sha256],
    ['sha512', 'SHA-512', details.checksum.sha512],
    ['md5', 'MD5', details.checksum.md5],
  ];
  return entries
    .filter((entry): entry is [string, string, string] => typeof entry[2] === 'string' && entry[2] !== '')
    .map(([id, label, value]) => ({
      id,
      label,
      mono: true,
      value: (
        <span className={styles.checksum}>
          <span className={styles.checksumText}>{value}</span>
          <CopyButton text={value} label={t('browse.details.copyChecksum', { name: label })} />
        </span>
      ),
    }));
}

export function AssetPanel({ repository, node, onNavigate }: PanelProps & { node: TreeNode }) {
  const t = useT();
  const { language } = useI18n();
  const toast = useToast();
  const changed = useContentChanged();
  const client = useQueryClient();
  const { state } = useSession();
  const key = assetPath(repository.name, node.path);
  const query = useApi<AssetDetails>(key);
  if (query.isPending) {
    return (
      <div className={styles.panel} aria-busy="true">
        <Skeleton height="64px" />
        <Skeleton height="240px" />
      </div>
    );
  }
  if (query.isError) {
    return <ErrorState error={query.error} onRetry={() => query.refetch()} />;
  }
  const details = query.data;
  const component = details.component;
  const attributes = formatAttributes(details.format, details);
  const copyUrl = async () => {
    try {
      await copyText(details.downloadUrl);
      toast.success(t('browse.details.urlCopied'));
    } catch {
      toast.error(t('browse.details.copyFailed'));
    }
  };
  const remove = async () => {
    await api.delete(path`/service/rest/v1/assets/${details.id}`);
    onNavigate(parentOf(node.path));
    client.removeQueries({ queryKey: apiKey(key) });
    await changed();
    toast.success(t('browse.details.assetDeleted', { name: node.name }));
  };
  const owner =
    details.repository === details.browsedRepository
      ? details.repository
      : t('browse.details.throughGroup', { member: details.repository, group: details.browsedRepository });
  const items: DetailItem[] = [
    { id: 'repository', label: t('browse.details.repository'), value: owner },
    {
      id: 'format',
      label: t('browse.details.format'),
      value: <FormatBadge format={details.format} />,
    },
  ];
  if (component) {
    items.push(
      ...(component.group && details.format !== 'raw'
        ? [{ id: 'group', label: t('browse.details.group'), value: component.group, mono: true }]
        : []),
      ...(details.format !== 'raw'
        ? [
            { id: 'name', label: t('browse.details.name'), value: component.name, mono: true },
            ...(component.version
              ? [{ id: 'version', label: t('browse.details.version'), value: component.version, mono: true }]
              : []),
          ]
        : []),
    );
  }
  items.push(
    { id: 'size', label: t('browse.details.size'), value: <FileSize bytes={details.fileSize} /> },
    { id: 'contentType', label: t('browse.details.contentType'), value: details.contentType, mono: true },
    { id: 'uploader', label: t('browse.details.uploader'), value: details.uploader },
    { id: 'created', label: t('browse.details.created'), value: <DateTime value={details.blobCreated} /> },
    { id: 'updated', label: t('browse.details.updated'), value: <DateTime value={details.blobUpdated} /> },
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
    { id: 'blobStore', label: t('browse.details.blobStore'), value: details.blobStoreName },
  );
  const fileName = node.path.split('/').pop() ?? node.name;
  return (
    <div className={styles.panel}>
      <PanelHeader
        icon={<File size={20} />}
        path={details.path}
        title={
          component && details.format === 'docker' ? (
            <VersionTitle
              name={componentTitle(details.format, component.group, component.name)}
              version={component.version}
            />
          ) : (
            fileName
          )
        }
        actions={
          <>
            <Button asChild variant="soft">
              <a href={details.downloadUrl} download={fileName}>
                <Download size={15} aria-hidden="true" />
                {t('browse.details.download')}
              </a>
            </Button>
            <Button variant="soft" color="gray" onClick={copyUrl}>
              <Link2 size={15} aria-hidden="true" />
              {t('browse.details.copyUrl')}
            </Button>
            {details.canDelete && (
              <DeleteButton
                label={t('browse.details.deleteAsset')}
                title={t('browse.details.deleteAssetTitle')}
                description={t('browse.details.deleteAssetText', {
                  name: details.path,
                  repository: details.repository,
                })}
                confirmLabel={t('browse.details.delete')}
                onConfirm={remove}
              />
            )}
          </>
        }
      />
      {supportsUsage(details.format) && (
        <Section title={t('browse.details.usage')}>
          <UsageSnippets
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
        </Section>
      )}
      <Section title={t('browse.details.details')}>
        <DetailList items={items} />
        {component && details.format !== 'raw' && (
          <Flex gap="2" wrap="wrap">
            <ComponentLink format={details.format} {...component} />
          </Flex>
        )}
      </Section>
      <Section title={t('browse.details.checksums')}>
        <DetailList items={checksumItems(details, t)} />
      </Section>
      {Object.keys(attributes).length > 0 && (
        <Section title={t('browse.details.attributes', { format: formatName(details.format) })}>
          <AttributeList
            attributes={attributes}
            label={t('browse.details.attributes', { format: formatName(details.format) })}
          />
        </Section>
      )}
    </div>
  );
}
