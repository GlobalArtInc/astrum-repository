

/**
 * Browse Detail Panel Components
 *
 * Components for displaying detailed information about selected items
 * in the browse tree (components, assets, and folders).
 */

// Main container component
export { DetailPanel, default } from './DetailPanel';

// Individual detail panels
export { ComponentDetailPanel } from './ComponentDetailPanel';
export { AssetDetailPanel } from './AssetDetailPanel';
export { FolderDetailPanel } from './FolderDetailPanel';

// Types
export type {
  BrowseNode,
  BrowseNodeType,
  ComponentXO,
  AssetXO,
  AssetAttributes,
  FolderInfo,
  DetailPanelSelection,
  DetailPanelProps,
  ComponentDetailPanelProps,
  AssetDetailPanelProps,
  FolderDetailPanelProps,
  DetailPanelStrings,
} from './detail.types';

// Utilities
export {
  formatFileSize,
  formatDate,
  formatRelativeDate,
  getAssetDownloadUrl,
  truncateText,
  getFilenameFromPath,
  isAssetCached,
  getLastDownloadedDisplay,
} from './detail.utils';

