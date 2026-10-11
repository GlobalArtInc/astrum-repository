import UploadStrings from '../../constants/pages/upload/UploadStrings';
import CommonStrings from '../../constants/CommonStrings';


/**
 * Represents a repository that supports file uploads.
 */
export interface UploadableRepository {
  /** The unique name/identifier of the repository */
  name: string;
  /** The format of the repository (e.g., 'maven2', 'npm', 'docker') */
  format: string;
  /** The URL to access the repository */
  url: string;
}

/**
 * Represents the status of a repository (online/offline).
 * Note: ExtDirect uses nested status object, REST uses flat online property.
 */
export interface RepositoryStatus {
  online: boolean;
  description?: string;
  reason?: string;
}

/**
 * Represents a repository reference from the REST API.
 * Note: REST API returns flat structure with `online` at top level,
 * while ExtDirect returns nested `status.online`.
 */
export interface RepositoryReference {
  name: string;
  type: 'hosted' | 'proxy' | 'group';
  format: string;
  url: string;
  /** REST API returns online directly (not nested in status) */
  online?: boolean;
  /** ExtDirect compatibility - may contain nested online status */
  status?: RepositoryStatus;
  versionPolicy?: string;
}

/**
 * Represents an upload definition from the API.
 */
export interface UploadDefinition {
  format: string;
  uiUpload: boolean;
  multipleUpload: boolean;
  componentFields?: UploadFieldDefinition[];
  assetFields?: UploadFieldDefinition[];
}

/**
 * Represents a field definition for the upload form.
 */
export interface UploadFieldDefinition {
  name: string;
  type: 'STRING' | 'BOOLEAN' | 'SELECT' | 'FILE';
  displayName?: string;
  helpText?: string;
  optional?: boolean;
  initialValue?: string | boolean;
  selectOptions?: string[];
}

/**
 * Sort direction for table columns.
 */
export type SortDirection = 'asc' | 'desc' | null;

/**
 * Sortable column names for the upload repository list.
 */
export type SortColumn = 'name' | 'format';

/**
 * State returned by useUploadableRepositories hook.
 */
export interface UploadableRepositoriesState {
  /** List of repositories that support upload */
  repositories: UploadableRepository[];
  /** Whether the data is currently loading */
  loading: boolean;
  /** Error message if fetch failed */
  error: string | null;
  /** Current filter text */
  filterText: string;
  /** Current sort column */
  sortColumn: SortColumn | null;
  /** Current sort direction */
  sortDirection: SortDirection;
}

/**
 * Props for the UploadRepositoryList component.
 */
export interface UploadRepositoryListProps {
  /** List of repositories to display */
  repositories: UploadableRepository[];
  /** Whether data is loading */
  loading: boolean;
  /** Error message to display */
  error: string | null;
  /** Current sort column */
  sortColumn: SortColumn | null;
  /** Current sort direction */
  sortDirection: SortDirection;
  /** Callback when a column header is clicked for sorting */
  onSort: (column: SortColumn) => void;
  /** Callback when a repository row is clicked */
  onSelect: (repositoryName: string) => void;
}

/**
 * UI strings for the upload module.
 */
export const UPLOAD_STRINGS = {
  pageTitle: UploadStrings.TYPES.UPLOAD,
  pageDescription: UploadStrings.TYPES.UPLOAD_CONTENT_HOSTED_REPOSITORIES,
  filterPlaceholder: UploadStrings.TYPES.FILTER_NAME,
  emptyMessage: UploadStrings.TYPES.NO_REPOSITORIES_FOUND,
  columns: {
    name: CommonStrings.LABELS.NAME,
    format: CommonStrings.LABELS.FORMAT,
    url: 'URL',
  },
  copyUrlTooltip: UploadStrings.TYPES.COPY_URL_CLIPBOARD,
  urlCopied: UploadStrings.TYPES.URL_COPIED_CLIPBOARD,
  loadingMessage: UploadStrings.TYPES.LOADING_REPOSITORIES,
  errorPrefix: UploadStrings.TYPES.ERROR_LOADING_REPOSITORIES,
};

/**
 * Extended upload definition with regex mapping support.
 */
export interface UploadDefinitionExtended extends UploadDefinition {
  regexMap?: {
    regex: string;
    fieldList: string[];
  };
}

/**
 * Repository settings from the API.
 */
export interface RepositorySettings {
  name: string;
  format: string;
  type: 'hosted' | 'proxy' | 'group';
  url: string;
  status?: RepositoryStatus;
  versionPolicy?: string;
}

/**
 * Field definition with group support (from componentFields).
 */
export interface UploadComponentField extends UploadFieldDefinition {
  group?: string;
}

/**
 * Asset field data (file + associated metadata).
 */
export interface AssetFieldData {
  file: File | null;
  [key: string]: string | boolean | File | null;
}

/**
 * Form data structure for upload.
 */
export interface UploadFormData {
  assets: AssetFieldData[];
  componentFields: Record<string, string | boolean>;
}

/**
 * Validation error structure.
 */
export interface ValidationErrors {
  assets?: Array<Record<string, string | null>>;
  componentFields?: Record<string, string | null>;
  general?: string;
}

/**
 * State returned by useUploadDefinition hook.
 */
export interface UseUploadDefinitionResult {
  loading: boolean;
  error: string | null;
  uploadDefinition: UploadDefinitionExtended | null;
  repositorySettings: RepositorySettings | null;
  componentFields: UploadComponentField[];
  componentFieldsByGroup: Record<string, UploadComponentField[]>;
  assetFields: UploadFieldDefinition[];
  multipleUpload: boolean;
  regexMap: UploadDefinitionExtended['regexMap'] | null;
  refetch: () => void;
}

/**
 * State returned by useUploadForm hook.
 */
export interface UseUploadFormResult {
  formData: UploadFormData;
  validationErrors: ValidationErrors;
  isSubmitting: boolean;
  isValid: boolean;
  isDirty: boolean;
  touchedFields: Set<string>;

  // File actions
  setAssetFile: (assetIndex: number, file: File | null) => void;
  setAssetField: (assetIndex: number, fieldName: string, value: string | boolean) => void;
  addAsset: () => void;
  removeAsset: (assetIndex: number) => void;

  // Component field actions
  setComponentField: (fieldName: string, value: string | boolean) => void;
  blurComponentField: (fieldName: string) => void;
  blurAssetField: (assetIndex: number, fieldName: string) => void;

  // Form actions
  validate: () => boolean;
  submit: () => Promise<{ success: boolean; error?: string; componentName?: string }>;
  reset: () => void;
}

/**
 * Upload form strings.
 */
export const UPLOAD_FORM_STRINGS = {
  backButton: UploadStrings.TYPES.BACK_UPLOAD,
  uploadTo: UploadStrings.TYPES.UPLOAD_TO,
  assetLabel: UploadStrings.TYPES.ASSET,
  fileRequired: UploadStrings.TYPES.FILE_REQUIRED,
  fieldRequired: UploadStrings.TYPES.FIELD_REQUIRED,
  assetNotUnique: UploadStrings.TYPES.ASSET_FIELDS_MUST_UNIQUE,
  addAsset: UploadStrings.TYPES.ADD_ANOTHER_ASSET,
  removeAsset: UploadStrings.TYPES.REMOVE_ASSET,
  uploadButton: UploadStrings.TYPES.UPLOAD,
  uploading: UploadStrings.TYPES.UPLOADING,
  uploadSuccess: UploadStrings.TYPES.UPLOAD_SUCCESSFUL,
  uploadError: UploadStrings.TYPES.UPLOAD_FAILED,
  coordinatesFromPom: UploadStrings.TYPES.COMPONENT_DETAILS_EXTRACTED_PROVIDED_POM,
  noDefinition: UploadStrings.TYPES.NO_UPLOAD_DEFINITION_FOUND_REPOSITORY,
  repositoryNotFound: UploadStrings.TYPES.REPOSITORY_NOT_FOUND,
};

// =============================================================================
// FORMAT-SPECIFIC FIELD TYPES (Agent 5 deliverables)
// =============================================================================

/**
 * Supported repository formats for specialized field rendering.
 */
export type SupportedFormat = 'maven2' | 'npm' | 'raw' | string;

/**
 * Props for the UploadFieldRenderer component.
 */
export interface UploadFieldRendererProps {
  /** Repository format (e.g., 'maven2', 'npm', 'raw') */
  format: SupportedFormat;
  /** Upload definition containing field metadata */
  definition: UploadDefinitionExtended | null;
  /** Current form values */
  values: Record<string, string | boolean>;
  /** Callback when a field value changes */
  onChange: (fieldName: string, value: string | boolean) => void;
  /** Validation errors by field name */
  errors: Record<string, string | null>;
  /** Fields that should be disabled */
  disabledFields?: Set<string>;
  /** Whether POM file is present (Maven-specific) */
  hasPomFile?: boolean;
  /** Optional pre-grouped fields */
  fieldsByGroup?: Record<string, UploadComponentField[]>;
}

/**
 * Props for individual field components.
 */
export interface UploadFieldProps {
  /** Field definition from API */
  field: UploadFieldDefinition | UploadComponentField;
  /** Current field value */
  value: string | boolean;
  /** Callback when value changes */
  onChange: (value: string | boolean) => void;
  /** Validation error message */
  error?: string | null;
  /** Whether field is disabled */
  disabled?: boolean;
}

/**
 * Props for format-specific field group components.
 */
export interface FormatFieldsProps {
  /** Component fields grouped by group name */
  fieldsByGroup: Record<string, UploadComponentField[]>;
  /** Current form values */
  values: Record<string, string | boolean>;
  /** Callback when a field value changes */
  onChange: (fieldName: string, value: string | boolean) => void;
  /** Validation errors by field name */
  errors: Record<string, string | null>;
  /** Fields that should be disabled */
  disabledFields?: Set<string>;
  /** Whether POM file is present (Maven-specific) */
  hasPomFile?: boolean;
}

/**
 * Maven-specific field names.
 */
export const MAVEN_FIELD_NAMES = {
  GROUP_ID: 'groupId',
  ARTIFACT_ID: 'artifactId',
  VERSION: 'version',
  PACKAGING: 'packaging',
  EXTENSION: 'extension',
  CLASSIFIER: 'classifier',
  GENERATE_POM: 'generate-pom',
} as const;

/**
 * Maven packaging options.
 */
export const MAVEN_PACKAGING_OPTIONS = [
  'jar',
  'war',
  'ear',
  'pom',
  'maven-plugin',
  'ejb',
  'bundle',
] as const;

/**
 * Format-specific field strings.
 */
export const FORMAT_FIELD_STRINGS = {
  maven: {
    groupName: UploadStrings.TYPES.COMPONENT_COORDINATES,
    groupIdLabel: UploadStrings.TYPES.GROUP_ID,
    groupIdHelp: UploadStrings.TYPES.MAVEN_GROUPID_E_G_COM,
    artifactIdLabel: UploadStrings.TYPES.ARTIFACT_ID,
    artifactIdHelp: UploadStrings.TYPES.MAVEN_ARTIFACTID_E_G_MY,
    versionLabel: UploadStrings.TYPES.VERSION,
    versionHelp: UploadStrings.TYPES.VERSION_NUMBER_E_G_1,
    packagingLabel: UploadStrings.TYPES.PACKAGING,
    packagingHelp: UploadStrings.TYPES.PACKAGING_TYPE,
    extensionLabel: UploadStrings.TYPES.EXTENSION,
    extensionHelp: UploadStrings.TYPES.FILE_EXTENSION_OPTIONAL_DEFAULTS_PACKAGING,
    classifierLabel: UploadStrings.TYPES.CLASSIFIER,
    classifierHelp: UploadStrings.TYPES.OPTIONAL_CLASSIFIER_E_G_SOURCES,
    generatePomLabel: UploadStrings.TYPES.GENERATE_POM,
    generatePomHelp: UploadStrings.TYPES.AUTOMATICALLY_GENERATE_POM_FILE,
    pomDetectedInfo: UploadStrings.TYPES.COMPONENT_COORDINATES_EXTRACTED_PROVIDED_POM,
  },
  npm: {
    packageNameLabel: UploadStrings.TYPES.PACKAGE_NAME,
    packageNameHelp: UploadStrings.TYPES.NPM_PACKAGE_NAME_E_G,
    versionLabel: UploadStrings.TYPES.VERSION,
    versionHelp: UploadStrings.TYPES.SEMVER_VERSION_E_G_1,
  },
  raw: {
    directoryLabel: UploadStrings.TYPES.DIRECTORY,
    directoryHelp: UploadStrings.TYPES.TARGET_DIRECTORY_PATH_E_G,
    filenameLabel: UploadStrings.TYPES.FILENAME,
    filenameHelp: UploadStrings.TYPES.FILENAME_OPTIONAL_USES_UPLOADED_FILENAME,
  },
  generic: {
    optionalSuffix: UploadStrings.TYPES.OPTIONAL_SUFFIX,
    requiredIndicator: '*',
    selectPlaceholder: UploadStrings.TYPES.SELECT,
  },
};


