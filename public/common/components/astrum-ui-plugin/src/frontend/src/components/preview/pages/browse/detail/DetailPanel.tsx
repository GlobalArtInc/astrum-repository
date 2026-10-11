

import React, { useState, useCallback, useEffect, useMemo } from 'react';
import {
  Box,
  Button,
  Callout,
  Card,
  DataList,
  Flex,
  Heading,
  Spinner,
  Text,
} from '@radix-ui/themes';
import {
  AlertCircle,
  ChevronDown,
  Download,
  ExternalLink,
  Trash2,
} from 'lucide-react';

import { ExtJS } from '../../../../../interface/ExtJS';
import { DeleteDialog } from '../actions/DeleteDialog';
import { deleteComponent, deleteAsset, deleteFolder, fetchBrowseNodes } from '../browse.api';
import type { DeleteItemInfo } from '../actions/actions.types';
import type { BrowseNode } from '../tree/browse-tree.types';
import { formatDateTime, getAssetDownloadUrl } from './detail.utils';
import { useToast } from '../../../shared';
import { CopyButton, UsageBlock } from '../../../shared/Usage';
import { FORMAT_LABELS } from '../../../shared/Badges/formatConstants';
import { isIqServerEnabled } from '../repository-list/useRepositoryList';
import {
  formatAttributeValue,
  shouldDisplayAttributeFacet,
} from '../browse.constants';
import { getUsageSnippets, type UsageContext } from '../../search/details/dependencySnippets/usage';
import { trackSnippetCopy } from '../../search/details/dependencySnippets/trackSnippetCopy';
import { formatBytes, formatNumber } from '../../Welcome/usage/usageFormat';
import { repositoryUrlOf, type RepositoryInfo } from '../useRepositoryInfo';
import UIStrings from '../../../constants/UIStrings';

import './DetailPanel.scss';

const DELETE_TIMEOUT_MS = 30_000;

function strings() {
  return UIStrings.BROWSE.DETAIL;
}

/**
 * Props for DetailPanel component.
 */
export interface DetailPanelProps {
  /** Currently selected node (null if none) */
  node: BrowseNode | null;
  /** Repository name for context */
  repositoryName: string;
  /** Format, type and URL of the repository, for the usage snippets */
  repository?: RepositoryInfo | null;
  /** Component details if available */
  componentData?: ComponentData | null;
  /** Asset details if available */
  assetData?: AssetData | null;
  /** Whether details are loading */
  loading?: boolean;
  /** Error message if loading failed */
  error?: string | null;
  /** Callback when item is deleted */
  onDeleted?: () => void;
  /** Whether user has delete permission */
  canDelete?: boolean;
  /** Tab from the URL; the panel shows every section at once, so it is only kept for old links */
  activeTab?: string;
  /** Tab change callback, kept for old links */
  onTabChange?: (tab: string) => void;
}

/**
 * Component data from the API.
 */
export interface ComponentData {
  id: string;
  repositoryName: string;
  format: string;
  group: string | null;
  name: string;
  version: string | null;
  /** Downloads of all assets of the component; undefined when not known */
  downloadCount?: number | null;
}

/**
 * Asset data from the API.
 */
export interface AssetData {
  id: string;
  name: string;
  format: string;
  contentType?: string;
  size?: number;
  repositoryName: string;
  blobCreated: string | null;
  blobUpdated: string | null;
  lastDownloaded: string | null;
  /** Number of times the asset was downloaded; undefined when not known */
  downloadCount?: number | null;
  path?: string;
  downloadUrl?: string;
  blobRef?: string;
  createdBy?: string;
  createdByIp?: string; // mapped from API but intentionally not rendered — privacy concern
  checksum?: Record<string, string>;
  /**
   * Format-specific attributes bag from the backend (e.g. `docker.registryUrl`).
   * Intentionally loose (`Record<string, unknown>`) since shape varies per format.
   */
  attributes?: Record<string, unknown>;
  registryUrl?: string;
}

// =============================================================================
// Helpers
// =============================================================================

const DOCKER_MANIFEST = /^\/?v2\/(.+)\/manifests\/([^/]+)$/;
const AT_SEPARATED_FORMATS = new Set(['npm', 'go']);
const CHECKSUM_LABELS: Record<string, string> = {
  md5: 'MD5',
  sha1: 'SHA-1',
  sha256: 'SHA-256',
  sha512: 'SHA-512',
};

function safeDecode(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function withLeadingSlash(path: string): string {
  return path.startsWith('/') ? path : `/${path}`;
}

/**
 * Title of the selected item: the component name with its version, `herald-api:0.1.0`, when there is
 * a component, else the name of the node. A Docker manifest stored under a digest has no component,
 * so its image and digest are read from the path.
 */
export function detailTitle(
  node: BrowseNode,
  format: string | undefined,
  componentData: ComponentData | null | undefined,
  path?: string | null,
): { name: string; version?: string; separator: string } {
  const docker = format === 'docker' && path ? DOCKER_MANIFEST.exec(path) : null;
  if (docker) {
    const reference = safeDecode(docker[2]);
    if (reference.startsWith('sha256:')) {
      return { name: docker[1], version: `sha256:${reference.substring(7, 19)}`, separator: '@' };
    }
    return { name: componentData?.name || docker[1], version: reference, separator: ':' };
  }
  if (componentData?.name && componentData.version) {
    const scoped = format === 'npm' && componentData.group;
    const name = scoped ? `@${componentData.group?.replace(/^@/, '')}/${componentData.name}` : componentData.name;
    return {
      name,
      version: componentData.version,
      separator: AT_SEPARATED_FORMATS.has(format ?? '') ? '@' : ':',
    };
  }
  return { name: node.text, separator: ':' };
}

function formatLabel(format: string | null | undefined): string | undefined {
  if (!format) return undefined;
  return FORMAT_LABELS[format] ?? format;
}

function countText(count: number | null | undefined): string {
  return typeof count === 'number' && count >= 0 ? formatNumber(count) : '—';
}

function sizeText(size: number | null | undefined): string {
  return typeof size === 'number' && size >= 0 ? formatBytes(size) : '—';
}

/**
 * The usage context of the selected item, or null when there is nothing to address.
 */
function usageContextOf(
  format: string | undefined,
  repositoryName: string,
  repository: RepositoryInfo | null | undefined,
  componentData: ComponentData | null | undefined,
  assetData: AssetData | null | undefined,
  path: string | null | undefined,
  fallbackName?: string,
): UsageContext | null {
  if (!format) {
    return null;
  }
  return {
    format,
    repositoryName,
    repositoryUrl: repositoryUrlOf(repositoryName, repository?.url),
    repositoryType: repository?.type,
    group: componentData?.group ?? null,
    name: componentData?.name ?? fallbackName ?? null,
    version: componentData?.version ?? null,
    path: path ?? null,
    registryUrl: assetData?.registryUrl ?? null,
    attributes: assetData?.attributes ?? null,
    distribution: repository?.distribution ?? null,
  };
}

// =============================================================================
// Building blocks
// =============================================================================

interface DetailRowData {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
}

/**
 * A two-column definition list of the details of the selected item.
 */
function DetailsList({ rows, testId }: { rows: DetailRowData[]; testId?: string }): JSX.Element {
  return (
    <DataList.Root
      size="2"
      orientation={{ initial: 'vertical', sm: 'horizontal' }}
      className="detail-panel__list"
      data-testid={testId}
    >
      {rows.map((row) => (
        <DataList.Item key={row.label} className="detail-panel__row">
          <DataList.Label minWidth="160px" className="detail-panel__label">{row.label}</DataList.Label>
          <DataList.Value className={row.mono ? 'detail-panel__value detail-panel__value--mono' : 'detail-panel__value'}>
            {row.value === null || row.value === undefined || row.value === '' ? '—' : row.value}
          </DataList.Value>
        </DataList.Item>
      ))}
    </DataList.Root>
  );
}

/**
 * The card that holds the details list.
 */
function DetailsCard({ rows, title }: { rows: DetailRowData[]; title?: string }): JSX.Element {
  return (
    <Card size="3" className="detail-panel__card" data-testid="detail-details">
      <Flex direction="column" gap="3">
        <Text as="div" size="3" weight="bold">{title ?? strings().DETAILS}</Text>
        <DetailsList rows={rows} />
      </Flex>
    </Card>
  );
}

/**
 * Head of the selected item: its path in mono above the title, and the actions on the right.
 */
function DetailHeader({
  path,
  title,
  actions,
}: {
  path?: string | null;
  title: { name: string; version?: string; separator: string };
  actions?: React.ReactNode;
}): JSX.Element {
  return (
    <Flex align="start" justify="between" gap="3" wrap="wrap" className="detail-panel__header">
      <Flex direction="column" gap="1" minWidth="0" className="detail-panel__heading">
        {path && (
          <Flex align="center" gap="1" minWidth="0" className="detail-panel__path-line">
            <Text size="1" color="gray" className="detail-panel__path" data-testid="detail-path">
              {path}
            </Text>
            <CopyButton text={path} iconOnly size="1" label={strings().COPY_PATH} />
          </Flex>
        )}
        <Heading as="h2" size="6" weight="bold" className="detail-panel__title">
          {title.name}
          {title.version && (
            <span className="detail-panel__title-version">{`${title.separator}${title.version}`}</span>
          )}
        </Heading>
      </Flex>
      {actions && (
        <Flex gap="2" wrap="wrap" className="detail-panel__actions">
          {actions}
        </Flex>
      )}
    </Flex>
  );
}

/**
 * The "Usage" block of an item, with the copy analytics of the Classic snippets.
 */
function ItemUsage({ context }: { context: UsageContext | null }): JSX.Element | null {
  const toast = useToast();
  const snippets = useMemo(() => (context ? getUsageSnippets(context) : []), [context]);
  if (!context || snippets.length === 0) {
    return null;
  }
  return (
    <UsageBlock
      snippets={snippets}
      onCopied={(snippet) => trackSnippetCopy(context.format, snippet.label)}
      onCopyFailed={() => toast.error(UIStrings.BROWSE.USAGE.COPY_FAILED)}
    />
  );
}

/**
 * Collapsible facet section for attribute groups (Firewall, Npm, Docker, etc.).
 * Default is closed; call sites explicitly pass defaultOpen for important facets.
 */
function FacetSection({
  title,
  children,
  defaultOpen = false,
  testId,
}: {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
  testId?: string;
}): JSX.Element {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <Box className="detail-panel__facet" data-testid={testId}>
      <button
        type="button"
        className="detail-panel__facet-toggle"
        onClick={() => setIsOpen((open) => !open)}
        aria-expanded={isOpen}
      >
        <Text size="2" weight="medium" className="detail-panel__facet-title">{title}</Text>
        <ChevronDown
          size={16}
          aria-hidden
          className={isOpen ? 'detail-panel__facet-chevron detail-panel__facet-chevron--open' : 'detail-panel__facet-chevron'}
        />
      </button>
      {isOpen && <Box pt="2">{children}</Box>}
    </Box>
  );
}

/**
 * Format attributes of an asset (Docker, npm, firewall...), one collapsible section per facet,
 * matching the Classic UI's AssetAttributes.
 */
function AttributesCard({ assetData }: { assetData: AssetData }): JSX.Element | null {
  const facets = Object.entries(assetData.attributes ?? {})
    .filter(([key, value]) => shouldDisplayAttributeFacet(key, value))
    .map(([key, value]) => [
      key,
      Object.entries(value as Record<string, unknown>).filter(([, item]) => item !== null && item !== undefined && item !== ''),
    ] as const)
    .filter(([, entries]) => entries.length > 0);

  if (facets.length === 0) {
    return null;
  }
  const dockerLabels: Record<string, string> = strings().DOCKER_ATTRIBUTES;

  return (
    <Card size="3" className="detail-panel__card" data-testid="detail-attributes">
      <Flex direction="column" gap="2">
        <Text as="div" size="3" weight="bold">{strings().ATTRIBUTES}</Text>
        {facets.map(([sectionKey, entries]) => (
          <FacetSection
            key={sectionKey}
            title={sectionKey}
            testId={`attr-section-${sectionKey}`}
            defaultOpen={sectionKey === 'firewall' || sectionKey === 'npm' || sectionKey === 'docker'}
          >
            <DataList.Root size="2" orientation={{ initial: 'vertical', sm: 'horizontal' }} className="detail-panel__list">
              {entries.map(([key, value]) => (
                <DataList.Item key={key} className="detail-panel__row" data-testid={`attr-${sectionKey}-${key}`}>
                  <DataList.Label minWidth="160px" className="detail-panel__label">
                    {sectionKey === 'docker' ? (dockerLabels[key] ?? key) : key}
                  </DataList.Label>
                  <DataList.Value className="detail-panel__value detail-panel__value--pre">
                    {formatAttributeValue(value, key, formatBytes)}
                  </DataList.Value>
                </DataList.Item>
              ))}
            </DataList.Root>
          </FacetSection>
        ))}
      </Flex>
    </Card>
  );
}

function DeleteButton({ label, onClick, color = 'red' }: { label: string; onClick: () => void; color?: 'red' | 'orange' }): JSX.Element {
  return (
    <Button variant="outline" color={color} size="2" onClick={onClick}>
      <Trash2 size={14} aria-hidden />
      {label}
    </Button>
  );
}

function researchUrl(format: string, componentData: ComponentData, referrer: string): string | null {
  if (!componentData.version) {
    return null;
  }
  const ecosystem = format === 'maven2' ? 'maven' : format;
  const componentPath =
    format === 'maven2' && componentData.group
      ? `${encodeURIComponent(componentData.group)}%3A${encodeURIComponent(componentData.name)}`
      : encodeURIComponent(componentData.name);
  return `https://guide.globalart.dev/component/${ecosystem}/${componentPath}/${encodeURIComponent(componentData.version)}?referrer=${referrer}`;
}

function ResearchButton({ format, componentData, referrer }: { format: string; componentData: ComponentData; referrer: string }): JSX.Element | null {
  const url = researchUrl(format, componentData, referrer);
  if (!url) {
    return null;
  }
  return (
    <Button variant="soft" color="gray" size="2" asChild>
      <a href={url} target="_blank" rel="noopener noreferrer">
        <ExternalLink size={14} aria-hidden />
        {strings().RESEARCH}
      </a>
    </Button>
  );
}

function IqServerButton({ componentData }: { componentData: ComponentData }): JSX.Element | null {
  if (!isIqServerEnabled()) {
    return null;
  }
  const iqUrl = ExtJS.state()?.getValue?.('clm')?.url;
  return (
    <Button
      variant="soft"
      color="gray"
      size="2"
      disabled={!iqUrl}
      onClick={() => {
        if (iqUrl && componentData.name) {
          window.open(
            `${iqUrl}/assets/index.html#/componentDetails/${encodeURIComponent(componentData.format)}/${encodeURIComponent(componentData.name)}/${encodeURIComponent(componentData.version || '')}`,
            '_blank',
            'noopener,noreferrer',
          );
        }
      }}
    >
      <ExternalLink size={14} aria-hidden />
      {strings().IQ_SERVER}
    </Button>
  );
}

// =============================================================================
// DetailPanel
// =============================================================================

/**
 * DetailPanel displays the details of a selected node in the browse tree.
 *
 * Features:
 * - Head with the path, the name and version, and the actions
 * - "Usage" block with a snippet per client, addressed through this repository
 * - Details list, format attributes
 * - Delete with confirmation (permission-gated), download for files
 */
export function DetailPanel({
  node,
  repositoryName,
  repository,
  componentData,
  assetData,
  loading = false,
  error = null,
  onDeleted,
  canDelete = false,
}: DetailPanelProps): JSX.Element {
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  /** When true, the pending delete targets the entire package folder; when false, just the metadata asset. */
  const [deleteAsFolder, setDeleteAsFolder] = useState(false);
  const toast = useToast();

  /**
   * Get the item info for delete dialog.
   * For package root nodes (asset + non-leaf), the target depends on `deleteAsFolder`:
   * - false → delete the metadata asset only
   * - true  → delete the entire package folder
   */
  const getDeleteItemInfo = useCallback((): DeleteItemInfo | null => {
    if (!node) return null;

    // Package root: asset node that is also a folder (has children)
    if (node.type === 'asset' && !node.leaf) {
      return deleteAsFolder
        ? { type: 'folder', id: node.id, name: node.text, repositoryName }
        : { type: 'asset', id: node.assetId || node.id, name: node.text, repositoryName };
    }

    switch (node.type) {
      case 'component':
        return {
          type: 'component',
          id: node.componentId || node.id,
          name: node.text,
          repositoryName,
        };
      case 'asset':
        return {
          type: 'asset',
          id: node.assetId || node.id,
          name: node.text,
          repositoryName,
        };
      case 'folder':
        return {
          type: 'folder',
          id: node.id,
          name: node.text,
          repositoryName,
        };
      default:
        return null;
    }
  }, [node, repositoryName, deleteAsFolder]);

  /**
   * Handle delete confirmation.
   * Uses Promise.race with a timeout to prevent indefinite hanging when the
   * backend returns a 500 (e.g., missing AptHostedMetadataFacet on cloud).
   * On error the tree is still refreshed because the delete may have partially
   * succeeded (asset removed but metadata update failed).
   */
  const handleDeleteConfirm = useCallback(async () => {
    setIsDeleting(true);
    setDeleteError(null);

    try {
      // Determine delete target. Order matters and MUST follow the selected node's type so the
      // action matches what the panel renders:
      // 1. Package root "delete entire package" (asset node that has children, deleteAsFolder=true)
      // 2. Standard folder delete (folder nodes were previously a no-op — this is the fix)
      // 3. Asset node -> delete ONLY that asset. The asset detail panel also loads componentData
      //    (to show component info), so we must key off node.type here; otherwise deleting a single
      //    asset would fall through to the componentData branch and cascade-delete the whole
      //    component. For multi-asset components (e.g. a Terraform provider version, which holds one
      //    zip per OS/arch) that wrongly removed every sibling platform.
      // 4. Component node -> delete the component.
      let deletePromise: Promise<unknown>;
      if (node && node.type === 'asset' && !node.leaf && deleteAsFolder) {
        deletePromise = deleteFolder(node.id, repositoryName);
      } else if (node && node.type === 'folder') {
        deletePromise = deleteFolder(node.id, repositoryName);
      } else if (node && node.type === 'asset' && assetData) {
        // assetData.id is the RAW internal id from ExtDirect readAsset (not the
        // base64 RepositoryItemIDXO the REST API expects); deleteAsset encodes it.
        deletePromise = deleteAsset(assetData.id, repositoryName);
      } else if (componentData) {
        deletePromise = deleteComponent({
          id: componentData.id,
          repositoryName: componentData.repositoryName,
          format: componentData.format,
          group: componentData.group,
          name: componentData.name,
          version: componentData.version,
        });
      } else if (assetData) {
        deletePromise = deleteAsset(assetData.id, repositoryName);
      } else {
        deletePromise = Promise.resolve();
      }

      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error(strings().DELETE_TIMEOUT)), DELETE_TIMEOUT_MS);
      });

      await Promise.race([deletePromise, timeoutPromise]);

      setDeleteDialogOpen(false);
      toast.success(strings().DELETED);
      if (onDeleted) {
        onDeleted();
      }
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : typeof err === 'string' ? err : '';
      toast.error(errorMessage || strings().DELETE_FAILED(''));
      setDeleteError(strings().DELETE_FAILED(errorMessage));
      setDeleteDialogOpen(false);

      // Refresh tree - delete may have partially succeeded on the server
      if (onDeleted) {
        onDeleted();
      }
    } finally {
      setIsDeleting(false);
    }
  }, [onDeleted, componentData, assetData, repositoryName, toast, node, deleteAsFolder]);

  /**
   * Open delete dialog and clear any previous delete error.
   */
  const handleDeleteClick = useCallback(() => {
    setDeleteError(null);
    setDeleteDialogOpen(true);
  }, []);

  /**
   * Handle download click.
   */
  const handleDownload = useCallback(() => {
    if (assetData) {
      const downloadUrl = assetData.downloadUrl ?? getAssetDownloadUrl(repositoryName, assetData.name);
      window.open(downloadUrl, '_blank');
    }
  }, [assetData, repositoryName]);

  if (!node) {
    return (
      <Card size="3" className="detail-panel detail-panel--empty" data-testid="detail-empty">
        <Flex direction="column" align="center" justify="center" gap="2" className="detail-panel__placeholder">
          <Text size="3" weight="medium">{strings().EMPTY_TITLE}</Text>
          <Text size="2" color="gray" align="center">{strings().EMPTY_DESCRIPTION}</Text>
        </Flex>
      </Card>
    );
  }

  if (loading) {
    return (
      <Card size="3" className="detail-panel detail-panel--loading">
        <Flex align="center" justify="center" direction="column" gap="3" className="detail-panel__placeholder">
          <Spinner size="3" />
          <Text color="gray" size="2">{strings().LOADING}</Text>
        </Flex>
      </Card>
    );
  }

  if (error) {
    return (
      <Callout.Root color="red" size="2" className="detail-panel detail-panel--error">
        <Callout.Icon>
          <AlertCircle size={16} />
        </Callout.Icon>
        <Callout.Text>{strings().ERROR(error)}</Callout.Text>
      </Callout.Root>
    );
  }

  const format = assetData?.format || componentData?.format || repository?.format;

  return (
    <Flex direction="column" gap="4" className="detail-panel">
      {deleteError && (
        <Callout.Root color="red" size="2">
          <Callout.Icon>
            <AlertCircle size={16} />
          </Callout.Icon>
          <Callout.Text>{deleteError}</Callout.Text>
        </Callout.Root>
      )}

      {node.type === 'component' && (
        <ComponentDetails
          node={node}
          format={format}
          repositoryName={repositoryName}
          repository={repository}
          componentData={componentData}
          canDelete={canDelete}
          onDeleteClick={handleDeleteClick}
        />
      )}

      {node.type === 'asset' && node.leaf && (
        <AssetDetails
          node={node}
          format={format}
          repositoryName={repositoryName}
          repository={repository}
          assetData={assetData}
          componentData={componentData}
          canDelete={canDelete}
          onDeleteClick={handleDeleteClick}
          onDownloadClick={handleDownload}
        />
      )}

      {node.type === 'asset' && !node.leaf && (
        <PackageRootDetails
          node={node}
          format={format}
          repositoryName={repositoryName}
          repository={repository}
          assetData={assetData}
          canDelete={canDelete}
          onDeleteMetadataClick={() => { setDeleteAsFolder(false); handleDeleteClick(); }}
          onDeleteFolderClick={() => { setDeleteAsFolder(true); handleDeleteClick(); }}
        />
      )}

      {node.type === 'folder' && (
        <FolderDetails
          node={node}
          repositoryName={repositoryName}
          canDelete={canDelete}
          onDeleteClick={handleDeleteClick}
        />
      )}

      <DeleteDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        item={getDeleteItemInfo()}
        onConfirm={handleDeleteConfirm}
        isDeleting={isDeleting}
      />
    </Flex>
  );
}

// =============================================================================
// Package root (npm): both a folder of versions and the package metadata
// =============================================================================

interface PackageRootDetailsProps {
  node: BrowseNode;
  format?: string;
  repositoryName: string;
  repository?: RepositoryInfo | null;
  assetData?: AssetData | null;
  canDelete: boolean;
  onDeleteMetadataClick: () => void;
  onDeleteFolderClick: () => void;
}

function PackageRootDetails({
  node,
  format,
  repositoryName,
  repository,
  assetData,
  canDelete,
  onDeleteMetadataClick,
  onDeleteFolderClick,
}: PackageRootDetailsProps): JSX.Element {
  const path = withLeadingSlash(assetData?.path || assetData?.name || node.id);
  const packageName = safeDecode(node.id);
  const scope = packageName.startsWith('@') ? packageName.split('/')[0].substring(1) : null;
  const name = scope ? packageName.split('/').slice(1).join('/') : packageName;
  const context = useMemo(
    () => usageContextOf(format, repositoryName, repository, { id: '', repositoryName, format: format ?? '', group: scope, name, version: null }, assetData, null),
    [format, repositoryName, repository, scope, name, assetData],
  );

  return (
    <>
      <Card size="3" className="detail-panel__card">
        <Flex direction="column" gap="5">
          <DetailHeader
            path={path}
            title={{ name: packageName, separator: ':' }}
            actions={canDelete ? (
              <>
                <DeleteButton label={strings().DELETE_METADATA} color="orange" onClick={onDeleteMetadataClick} />
                <DeleteButton label={strings().DELETE_PACKAGE} onClick={onDeleteFolderClick} />
              </>
            ) : undefined}
          />
          {canDelete && (
            <Text as="p" size="2" color="gray" className="detail-panel__note">{strings().PACKAGE_ROOT_NOTE}</Text>
          )}
          <ItemUsage context={context} />
        </Flex>
      </Card>
      <DetailsCard
        rows={[
          { label: strings().ROWS.REPOSITORY, value: repositoryName },
          { label: strings().ROWS.FORMAT, value: formatLabel(assetData?.format || format) },
          { label: strings().ROWS.CONTENT_TYPE, value: assetData?.contentType, mono: true },
          { label: strings().ROWS.SIZE, value: sizeText(assetData?.size) },
          { label: strings().ROWS.UPDATED, value: formatDateTime(assetData?.blobUpdated) },
          { label: UIStrings.BROWSE_DETAIL.DOWNLOADS, value: countText(assetData?.downloadCount) },
        ]}
      />
    </>
  );
}

// =============================================================================
// Component
// =============================================================================

interface ComponentDetailsProps {
  node: BrowseNode;
  format?: string;
  repositoryName: string;
  repository?: RepositoryInfo | null;
  componentData?: ComponentData | null;
  canDelete: boolean;
  onDeleteClick: () => void;
}

function ComponentDetails({
  node,
  format,
  repositoryName,
  repository,
  componentData,
  canDelete,
  onDeleteClick,
}: ComponentDetailsProps): JSX.Element {
  const path = withLeadingSlash(safeDecode(node.id));
  const title = detailTitle(node, format, componentData);
  const context = useMemo(
    () => usageContextOf(format, repositoryName, repository, componentData, null, componentData ? null : node.id, node.text),
    [format, repositoryName, repository, componentData, node.id, node.text],
  );

  return (
    <>
      <Card size="3" className="detail-panel__card">
        <Flex direction="column" gap="5">
          <DetailHeader
            path={path}
            title={title}
            actions={(
              <>
                {componentData && format && (
                  <ResearchButton format={format} componentData={componentData} referrer="browse-tree-component" />
                )}
                {canDelete && <DeleteButton label={strings().DELETE_COMPONENT} onClick={onDeleteClick} />}
              </>
            )}
          />
          <ItemUsage context={context} />
        </Flex>
      </Card>
      <DetailsCard
        rows={[
          { label: strings().ROWS.REPOSITORY, value: repositoryName },
          { label: strings().ROWS.FORMAT, value: formatLabel(componentData?.format || format) },
          ...(componentData?.group ? [{ label: strings().ROWS.GROUP, value: componentData.group, mono: true }] : []),
          { label: strings().ROWS.NAME, value: componentData?.name || node.text },
          { label: strings().ROWS.VERSION, value: componentData?.version, mono: true },
          { label: UIStrings.BROWSE_DETAIL.DOWNLOADS, value: countText(componentData?.downloadCount) },
        ]}
      />
    </>
  );
}

// =============================================================================
// Asset
// =============================================================================

interface AssetDetailsProps {
  node: BrowseNode;
  format?: string;
  repositoryName: string;
  repository?: RepositoryInfo | null;
  assetData?: AssetData | null;
  componentData?: ComponentData | null;
  canDelete: boolean;
  onDeleteClick: () => void;
  onDownloadClick: () => void;
}

function AssetDetails({
  node,
  format,
  repositoryName,
  repository,
  assetData,
  componentData,
  canDelete,
  onDeleteClick,
  onDownloadClick,
}: AssetDetailsProps): JSX.Element {
  const rawPath = assetData?.path || assetData?.name || node.id;
  const path = withLeadingSlash(rawPath);
  const title = detailTitle(node, format, componentData, path);
  const context = useMemo(
    () => usageContextOf(format, repositoryName, repository, componentData, assetData, rawPath),
    [format, repositoryName, repository, componentData, assetData, rawPath],
  );

  const checksums = Object.entries(assetData?.checksum ?? {});
  const isDocker = format === 'docker';
  const checksumRows: DetailRowData[] = checksums
    .filter(([algorithm]) => !(isDocker && algorithm === 'sha256'))
    .map(([algorithm, value]) => ({ label: CHECKSUM_LABELS[algorithm] ?? algorithm.toUpperCase(), value, mono: true }));
  const digest = isDocker && assetData?.checksum?.sha256 ? `sha256:${assetData.checksum.sha256}` : null;

  const rows: DetailRowData[] = [
    { label: strings().ROWS.REPOSITORY, value: repositoryName },
    { label: strings().ROWS.FORMAT, value: formatLabel(assetData?.format || format) },
    ...(componentData?.group ? [{ label: strings().ROWS.GROUP, value: componentData.group, mono: true }] : []),
    ...(componentData ? [{ label: strings().ROWS.COMPONENT, value: componentData.name }] : []),
    ...(componentData?.version ? [{ label: strings().ROWS.VERSION, value: componentData.version, mono: true }] : []),
    ...(assetData?.createdBy ? [{ label: strings().ROWS.UPLOADER, value: assetData.createdBy }] : []),
    { label: strings().ROWS.CONTENT_TYPE, value: assetData?.contentType, mono: true },
    { label: strings().ROWS.SIZE, value: sizeText(assetData?.size) },
    { label: strings().ROWS.CREATED, value: formatDateTime(assetData?.blobCreated) },
    ...(assetData?.blobUpdated ? [{ label: strings().ROWS.UPDATED, value: formatDateTime(assetData.blobUpdated) }] : []),
    {
      label: strings().ROWS.LAST_DOWNLOADED,
      value: assetData?.lastDownloaded ? formatDateTime(assetData.lastDownloaded) : strings().ROWS.NEVER_DOWNLOADED,
    },
    { label: UIStrings.BROWSE_DETAIL.DOWNLOADS, value: countText(assetData?.downloadCount) },
    ...(digest ? [{ label: strings().ROWS.DIGEST, value: digest, mono: true }] : []),
    ...checksumRows,
    ...(assetData?.blobRef ? [{ label: strings().ROWS.BLOB_REF, value: assetData.blobRef, mono: true }] : []),
  ];

  return (
    <>
      <Card size="3" className="detail-panel__card">
        <Flex direction="column" gap="5">
          <DetailHeader
            path={path}
            title={title}
            actions={(
              <>
                {componentData && format && (
                  <ResearchButton format={format} componentData={componentData} referrer="repo-browse-asset" />
                )}
                {componentData && <IqServerButton componentData={componentData} />}
                <Button variant="soft" color="gray" size="2" onClick={onDownloadClick}>
                  <Download size={14} aria-hidden />
                  {strings().DOWNLOAD}
                </Button>
                {canDelete && <DeleteButton label={strings().DELETE_FILE} onClick={onDeleteClick} />}
              </>
            )}
          />
          <ItemUsage context={context} />
        </Flex>
      </Card>
      <DetailsCard rows={rows} />
      {assetData && <AttributesCard assetData={assetData} />}
    </>
  );
}

// =============================================================================
// Folder
// =============================================================================

interface FolderDetailsProps {
  node: BrowseNode;
  repositoryName: string;
  canDelete: boolean;
  onDeleteClick: () => void;
}

interface FolderChildInfo {
  total: number;
  folders: number;
  components: number;
  assets: number;
}

function FolderDetails({
  node,
  repositoryName,
  canDelete,
  onDeleteClick,
}: FolderDetailsProps): JSX.Element {
  const [childInfo, setChildInfo] = useState<FolderChildInfo | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchBrowseNodes({ repositoryName, node: node.id })
      .then((children) => {
        if (cancelled) return;
        setChildInfo({
          total: children.length,
          folders: children.filter((c) => c.type === 'folder').length,
          components: children.filter((c) => c.type === 'component').length,
          assets: children.filter((c) => c.type === 'asset').length,
        });
      })
      .catch(() => {
        if (!cancelled) setChildInfo(null);
      });
    return () => { cancelled = true; };
  }, [repositoryName, node.id]);

  const depthSegments = node.id.split('/').filter(Boolean);
  const rows: DetailRowData[] = [
    { label: strings().ROWS.REPOSITORY, value: repositoryName },
    { label: strings().ROWS.PATH, value: withLeadingSlash(safeDecode(node.id)), mono: true },
    { label: strings().ROWS.DEPTH, value: String(depthSegments.length) },
  ];
  if (childInfo) {
    rows.push({ label: strings().ROWS.CHILDREN, value: formatNumber(childInfo.total) });
    if (childInfo.folders > 0) rows.push({ label: strings().ROWS.FOLDERS, value: formatNumber(childInfo.folders) });
    if (childInfo.components > 0) rows.push({ label: strings().ROWS.COMPONENTS, value: formatNumber(childInfo.components) });
    if (childInfo.assets > 0) rows.push({ label: strings().ROWS.ASSETS, value: formatNumber(childInfo.assets) });
  }

  return (
    <>
      <Card size="3" className="detail-panel__card">
        <Flex direction="column" gap="3">
          <DetailHeader
            path={withLeadingSlash(safeDecode(node.id))}
            title={{ name: node.text, separator: ':' }}
            actions={canDelete ? <DeleteButton label={strings().DELETE_FOLDER} onClick={onDeleteClick} /> : undefined}
          />
          {canDelete && (
            <Text as="p" size="2" color="gray" className="detail-panel__note">{strings().DELETE_FOLDER_WARNING}</Text>
          )}
        </Flex>
      </Card>
      <DetailsCard rows={rows} />
    </>
  );
}

export default DetailPanel;
