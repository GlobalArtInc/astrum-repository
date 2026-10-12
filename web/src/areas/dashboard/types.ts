export const DASHBOARD_PATH = '/service/rest/internal/ui/dashboard';

export interface TypeCounts {
  hosted: number;
  proxy: number;
  group: number;
}

export interface FormatCount extends TypeCounts {
  format: string;
  total: number;
}

export interface TopAsset {
  repository: string;
  format: string;
  path: string;
  assetId: string;
  componentId: string | null;
  downloads: number;
  lastDownloaded: string | null;
}

export interface TopComponent {
  repository: string;
  format: string;
  group: string;
  name: string;
  version: string;
  componentId: string;
  downloads: number;
}

export interface Upload {
  kind: 'component' | 'asset';
  repository: string;
  format: string;
  componentId?: string;
  group?: string;
  name: string;
  version?: string;
  path: string;
  assetId: string;
  assetCount: number;
  uploadedAt: string;
  uploader: string;
}

export interface StoreUsage {
  name: string;
  type: string;
  unavailable: boolean;
  blobCount: number;
  totalSize: number;
  availableSpace: number | null;
}

export interface Dashboard {
  repositories: { total: number; byType: TypeCounts; byFormat: FormatCount[] };
  components: number;
  assets: number;
  totalSize: number;
  downloads: { total: number; topAssets: TopAsset[]; topComponents: TopComponent[] };
  recentUploads: Upload[];
  blobStores?: StoreUsage[];
}

export function share(value: number, max: number): number {
  if (max <= 0 || value <= 0) {
    return 0;
  }
  return Math.max(2, Math.min(100, (value / max) * 100));
}

export function fileName(path: string): string {
  const parts = path.split('/').filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

export function storeUse(store: StoreUsage): number | null {
  if (store.availableSpace === null) {
    return null;
  }
  const total = store.totalSize + store.availableSpace;
  return total <= 0 ? 0 : (store.totalSize / total) * 100;
}
