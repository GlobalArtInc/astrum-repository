

/**
 * Upload Fields Module
 *
 * Format-specific field renderers for the upload form.
 *
 * Usage:
 * ```tsx
 * import { UploadFieldRenderer } from './components/fields';
 *
 * <UploadFieldRenderer
 *   format="maven2"
 *   definition={uploadDefinition}
 *   values={formValues}
 *   onChange={handleFieldChange}
 *   errors={fieldErrors}
 * />
 * ```
 */

// Main renderer
export { UploadFieldRenderer } from './UploadFieldRenderer';
export { default } from './UploadFieldRenderer';

// Format-specific components
export { GenericUploadFields, UploadField } from './GenericUploadFields';
export { MavenUploadFields } from './MavenUploadFields';
export { NpmUploadFields } from './NpmUploadFields';
export { RawUploadFields } from './RawUploadFields';

// Re-export types
export type {
  UploadFieldRendererProps,
  UploadFieldProps,
  FormatFieldsProps,
} from '../../upload.types';

