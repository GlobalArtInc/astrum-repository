import type { RepositoryType } from '../../api';

export const UI_PATH = '/service/rest/internal/ui';
export const REPOSITORIES_PATH = `${UI_PATH}/repositories`;

export interface RepositoryFlags {
  browse: boolean;
  read: boolean;
  add: boolean;
  edit: boolean;
  delete: boolean;
  admin: boolean;
  adminEdit: boolean;
  adminDelete: boolean;
}

export interface RepositoryItem {
  name: string;
  format: string;
  type: RepositoryType;
  url: string;
  online: boolean;
  blobStoreName: string | null;
  remoteUrl: string | null;
  members?: string[];
  memberOf: string[];
  componentCount: number | null;
  assetCount: number | null;
  totalSize: number | null;
  lastUploaded: string | null;
  limited: boolean;
  permissions: RepositoryFlags;
}

export interface Coordinates {
  group: string;
  name: string;
  version: string;
}

export type NodeKind = 'folder' | 'component' | 'asset';

export interface TreeNode {
  kind: NodeKind;
  name: string;
  path: string;
  childCount?: number;
  assetId?: string;
  componentId?: string;
  component?: Coordinates;
  repository?: string;
  size?: number;
  contentType?: string;
  lastModified?: string;
  downloadUrl?: string;
}

export interface TreeListing {
  repository: string;
  format: string;
  type: RepositoryType;
  path: string;
  truncated: boolean;
  children: TreeNode[];
}

export interface Checksums {
  md5?: string | null;
  sha1?: string | null;
  sha256?: string | null;
  sha512?: string | null;
}

export interface AssetComponent extends Coordinates {
  id: string;
  repository: string;
}

export interface AssetDetails {
  id: string;
  path: string;
  repository: string;
  browsedRepository: string;
  format: string;
  downloadUrl: string;
  checksum: Checksums;
  contentType: string;
  lastModified: string;
  lastDownloaded: string | null;
  uploader: string;
  uploaderIp: string | null;
  fileSize: number;
  blobCreated: string;
  blobUpdated: string;
  blobStoreName: string;
  lastVerified: string | null;
  downloadCount: number;
  component: AssetComponent | null;
  canDelete: boolean;
  [format: string]: unknown;
}

export interface SearchAsset {
  id: string;
  path: string;
  repository: string;
  format: string;
  downloadUrl: string;
  checksum: Checksums;
  contentType: string;
  lastModified: string;
  lastDownloaded: string | null;
  uploader: string;
  fileSize: number;
  downloadCount: number;
  [format: string]: unknown;
}

export interface SearchComponent {
  id: string;
  repository: string;
  format: string;
  group: string;
  name: string;
  version: string;
  assets: SearchAsset[];
}

export interface SearchPage {
  items: SearchComponent[];
  continuationToken: string | null;
}

export interface VersionEntry {
  version: string;
  repositories: string[];
  componentIds: string[];
  lastModified: string | null;
}

export interface VersionsResponse {
  format: string;
  group: string;
  name: string;
  versions: VersionEntry[];
}

export interface FolderDeleted {
  deletedAssets: number;
  deletedComponents: number;
  skippedAssets: number;
}

export function formatAttributes(format: string, value: Record<string, unknown>): Record<string, unknown> {
  const attributes = value[format];
  return attributes && typeof attributes === 'object' && !Array.isArray(attributes)
    ? (attributes as Record<string, unknown>)
    : {};
}
