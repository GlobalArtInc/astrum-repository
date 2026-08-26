

/**
 * Upload Module - Public API
 *
 * This is the main entry point for the Upload module.
 * Import types, components, and hooks from here.
 *
 * @example
 * import { UploadPage, useUploadableRepositories } from '../upload';
 */

// =============================================================================
// TYPES
// =============================================================================
export * from './upload.types';

// =============================================================================
// API
// =============================================================================
export {
  uploadToRepository,
  buildUploadFormData,
  calculateFormDataSize,
  formatBytes,
  formatTime,
  type UploadProgress,
  type UploadParams,
  type UploadResult,
  type UploadResponse,
} from './upload.api';

// =============================================================================
// HOOKS
// =============================================================================
export { useUploadableRepositories } from './hooks/useUploadableRepositories';
export { useUploadDefinition } from './hooks/useUploadDefinition';
export { useUploadForm } from './hooks/useUploadForm';
export {
  useUploadSubmit,
  UPLOAD_SUBMIT_STRINGS,
  type UploadSubmitParams,
  type UseUploadSubmitOptions,
  type UseUploadSubmitResult,
} from './hooks/useUploadSubmit';

// =============================================================================
// COMPONENTS
// =============================================================================
export { UploadRepositoryList } from './components/UploadRepositoryList';
export { FileDropzone } from './components/FileDropzone';
export { UploadPage } from './UploadPage';
export { UploadFormContainer } from './UploadFormContainer';
export { UploadRepositoryListPage } from './UploadRepositoryListPage';
export { UploadForm } from './UploadForm';
export { FileUploadZone } from './FileUploadZone';
export { RepositorySelector } from './RepositorySelector';
export { UploadFieldRenderer } from './UploadFieldRenderer';
export { default } from './UploadPage';

// =============================================================================
// FIELD COMPONENTS
// =============================================================================
export {
  GenericUploadFields,
  MavenUploadFields,
  NpmUploadFields,
  RawUploadFields,
} from './components/fields';
