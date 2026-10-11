import ContentSelectorsStrings from '../../../../constants/pages/settings/repository/ContentSelectorsStrings';
import CommonStrings from '../../../../constants/CommonStrings';


/**
 * Content selector data model
 */
export interface ContentSelector {
  name: string;
  type: string;
  description: string;
  expression: string;
}

/**
 * Repository option for preview
 */
export interface RepositoryOption {
  id: string;
  name: string;
}

/**
 * Content selector form data for create/edit
 */
export interface ContentSelectorFormData {
  name: string;
  type: string;
  description: string;
  expression: string;
}

/**
 * Form validation errors
 */
export interface ContentSelectorFormErrors {
  name?: string;
  description?: string;
  expression?: string;
}

/**
 * API URLs
 */
export const CONTENT_SELECTOR_API = {
  BASE_URL: 'service/rest/v1/security/content-selectors',
  PREVIEW_URL: 'service/rest/internal/ui/content-selectors/preview',
  // Must include withAll=true to get "All Repositories" option, withFormats=true for format info
  REPOSITORIES_URL: 'service/rest/internal/ui/repositories?withAll=true&withFormats=true',
};

/**
 * Get URL for a single content selector
 */
export const getContentSelectorUrl = (name: string): string =>
  `${CONTENT_SELECTOR_API.BASE_URL}/${encodeURIComponent(name)}`;

/**
 * Content selector type - only CSEL is supported
 */
export const CONTENT_SELECTOR_TYPE = 'csel';

/**
 * Empty content selector for create
 */
export const EMPTY_CONTENT_SELECTOR: ContentSelectorFormData = {
  name: '',
  type: CONTENT_SELECTOR_TYPE,
  description: '',
  expression: '',
};

/**
 * Maximum length for name field
 */
export const NAME_MAX_LENGTH = 512;

/**
 * CSEL Expression Language examples and help
 */
export const CSEL_HELP = {
  TITLE: ContentSelectorsStrings.TYPES.CONTENT_SELECTOR_EXPRESSION_LANGUAGE_CSEL,
  DESCRIPTION:
    ContentSelectorsStrings.TYPES.CSEL_ALLOWS_YOU_DEFINE_EXPRESSIONS,
  EXAMPLES: [
    {
      label: ContentSelectorsStrings.TYPES.SELECT_RAW_FORMAT_CONTENT,
      expression: 'format == "raw"',
    },
    {
      label: ContentSelectorsStrings.TYPES.SELECT_MAVEN2_CONTENT_ALONG_PATH,
      expression: 'format == "maven2" and path =^ "/org"',
    },
    {
      label: ContentSelectorsStrings.TYPES.SELECT_CONTENT_REPOSITORY_NAME,
      expression: 'coordinate.repositoryName == "maven-central"',
    },
    {
      label: ContentSelectorsStrings.TYPES.SELECT_CONTENT_PATH_PATTERN,
      expression: 'path =~ ".*\\\\.jar$"',
    },
  ],
  OPERATORS: [
    { operator: '==', description: ContentSelectorsStrings.TYPES.EQUALS },
    { operator: '!=', description: ContentSelectorsStrings.TYPES.NOT_EQUALS },
    { operator: '=~', description: ContentSelectorsStrings.TYPES.REGULAR_EXPRESSION_MATCH },
    { operator: '=^', description: ContentSelectorsStrings.TYPES.STARTS_WITH },
    { operator: 'and', description: ContentSelectorsStrings.TYPES.LOGICAL_AND },
    { operator: 'or', description: ContentSelectorsStrings.TYPES.LOGICAL_OR },
    { operator: 'not', description: ContentSelectorsStrings.TYPES.LOGICAL_NOT },
  ],
  ATTRIBUTES: [
    { attribute: 'format', description: ContentSelectorsStrings.TYPES.REPOSITORY_FORMAT_E_G_MAVEN2 },
    { attribute: 'path', description: ContentSelectorsStrings.TYPES.ASSET_PATH_WITHIN_REPOSITORY },
    { attribute: 'coordinate.repositoryName', description: ContentSelectorsStrings.TYPES.NAME_REPOSITORY },
    { attribute: 'coordinate.groupId', description: ContentSelectorsStrings.TYPES.MAVEN_GROUP_ID_MAVEN2_ONLY },
    { attribute: 'coordinate.artifactId', description: ContentSelectorsStrings.TYPES.MAVEN_ARTIFACT_ID_MAVEN2_ONLY },
    { attribute: 'coordinate.version', description: ContentSelectorsStrings.TYPES.COMPONENT_VERSION },
  ],
};

/**
 * Validate content selector name
 */
export const validateName = (name: string): string | undefined => {
  if (!name.trim()) {
    return CommonStrings.MESSAGES.NAME_REQUIRED;
  }
  if (name.length > NAME_MAX_LENGTH) {
    return ContentSelectorsStrings.TYPES.NAME_MUST_CHARACTERS_LESS(NAME_MAX_LENGTH);
  }
  if (!/^[a-zA-Z0-9\-_.]+$/.test(name)) {
    return ContentSelectorsStrings.TYPES.NAME_MUST_CONTAIN_ONLY_LETTERS;
  }
  return undefined;
};

/**
 * Validate content selector expression
 */
export const validateExpression = (expression: string): string | undefined => {
  if (!expression.trim()) {
    return ContentSelectorsStrings.TYPES.EXPRESSION_REQUIRED;
  }
  return undefined;
};

