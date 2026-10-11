import ContentSelectorsStrings from '../../../../constants/pages/settings/repository/ContentSelectorsStrings';


/**
 * CSEL (Content Selector Expression Language) Configuration
 *
 * This static configuration file defines the attributes, operators, and examples
 * available for CSEL expressions. It is UI-owned and versioned with the frontend.
 *
 * NO BACKEND CHANGES REQUIRED - This is purely frontend configuration.
 */

export interface CSELAttribute {
  name: string;
  description: string;
  operators: string[];
  formatSpecific?: string; // e.g., "maven2 only"
}

export interface CSELOperator {
  symbol: string;
  description: string;
  isLogical?: boolean;
}

export interface CSELExample {
  label: string;
  expression: string;
  description?: string;
}

export interface CSELConfig {
  attributes: CSELAttribute[];
  operators: CSELOperator[];
  logicalOperators: CSELOperator[];
  examples: CSELExample[];
}

/**
 * Static CSEL configuration
 */
export const CSEL_CONFIG: CSELConfig = {
  attributes: [
    {
      name: 'format',
      description: ContentSelectorsStrings.CSEL.REPOSITORY_FORMAT_E_G_MAVEN2,
      operators: ['==', '!=', '=~'],
    },
    {
      name: 'path',
      description: ContentSelectorsStrings.CSEL.ASSET_PATH_WITHIN_REPOSITORY,
      operators: ['==', '!=', '=~', '=^'],
    },
    {
      name: 'coordinate.repositoryName',
      description: ContentSelectorsStrings.CSEL.NAME_REPOSITORY,
      operators: ['==', '!='],
    },
    {
      name: 'coordinate.groupId',
      description: ContentSelectorsStrings.CSEL.MAVEN_GROUP_ID,
      operators: ['==', '!=', '=~'],
      formatSpecific: 'maven2 only',
    },
    {
      name: 'coordinate.artifactId',
      description: ContentSelectorsStrings.CSEL.MAVEN_ARTIFACT_ID,
      operators: ['==', '!=', '=~'],
      formatSpecific: 'maven2 only',
    },
    {
      name: 'coordinate.version',
      description: ContentSelectorsStrings.CSEL.COMPONENT_VERSION,
      operators: ['==', '!=', '=~'],
    },
    {
      name: 'coordinate.extension',
      description: ContentSelectorsStrings.CSEL.FILE_EXTENSION_E_G_JAR,
      operators: ['==', '!=', '=~'],
      formatSpecific: 'maven2 only',
    },
    {
      name: 'coordinate.classifier',
      description: ContentSelectorsStrings.CSEL.MAVEN_CLASSIFIER_E_G_SOURCES,
      operators: ['==', '!=', '=~'],
      formatSpecific: 'maven2 only',
    },
  ],

  operators: [
    { symbol: '==', description: ContentSelectorsStrings.CSEL.EQUALS },
    { symbol: '!=', description: ContentSelectorsStrings.CSEL.NOT_EQUALS },
    { symbol: '=~', description: ContentSelectorsStrings.CSEL.REGULAR_EXPRESSION_MATCH },
    { symbol: '=^', description: ContentSelectorsStrings.CSEL.STARTS_WITH },
  ],

  logicalOperators: [
    { symbol: 'and', description: ContentSelectorsStrings.CSEL.LOGICAL_AND, isLogical: true },
    { symbol: 'or', description: ContentSelectorsStrings.CSEL.LOGICAL_OR, isLogical: true },
    { symbol: 'not', description: ContentSelectorsStrings.CSEL.LOGICAL_NOT, isLogical: true },
  ],

  examples: [
    // Simple format selectors - safe to use
    {
      label: ContentSelectorsStrings.CSEL.RAW_FORMAT,
      expression: 'format == "raw"',
      description: ContentSelectorsStrings.CSEL.MATCHES_ALL_CONTENT_RAW_REPOSITORIES,
    },
    {
      label: ContentSelectorsStrings.CSEL.MAVEN_FORMAT,
      expression: 'format == "maven2"',
      description: ContentSelectorsStrings.CSEL.MATCHES_ALL_MAVEN_ARTIFACTS,
    },
    {
      label: ContentSelectorsStrings.CSEL.NPM_FORMAT,
      expression: 'format == "npm"',
      description: ContentSelectorsStrings.CSEL.MATCHES_ALL_NPM_PACKAGES,
    },
    {
      label: ContentSelectorsStrings.CSEL.DOCKER_FORMAT,
      expression: 'format == "docker"',
      description: ContentSelectorsStrings.CSEL.MATCHES_ALL_DOCKER_IMAGES,
    },
    // Path-based selectors - work with any format
    {
      label: ContentSelectorsStrings.CSEL.PATH_STARTS_ORG,
      expression: 'path =^ "/org"',
      description: ContentSelectorsStrings.CSEL.MATCHES_CONTENT_PATHS_STARTING_ORG,
    },
    // Combined expressions (from Default UI documentation)
    {
      label: ContentSelectorsStrings.CSEL.MAVEN_PATH_FILTER,
      expression: 'format == "maven2" and path =^ "/org"',
      description: ContentSelectorsStrings.CSEL.MATCHES_MAVEN_ARTIFACTS_PATHS_STARTING,
    },
  ],
};

/**
 * Get attribute by name
 */
export function getAttributeByName(name: string): CSELAttribute | undefined {
  return CSEL_CONFIG.attributes.find((attr) => attr.name === name);
}

/**
 * Get all attribute names
 */
export function getAttributeNames(): string[] {
  return CSEL_CONFIG.attributes.map((attr) => attr.name);
}

/**
 * Get operators for a specific attribute
 */
export function getOperatorsForAttribute(attributeName: string): string[] {
  const attribute = getAttributeByName(attributeName);
  return attribute?.operators || CSEL_CONFIG.operators.map((op) => op.symbol);
}

/**
 * Check if an attribute is known
 */
export function isKnownAttribute(name: string): boolean {
  return CSEL_CONFIG.attributes.some((attr) => attr.name === name);
}

/**
 * Check if an operator is valid
 */
export function isValidOperator(symbol: string): boolean {
  return (
    CSEL_CONFIG.operators.some((op) => op.symbol === symbol) ||
    CSEL_CONFIG.logicalOperators.some((op) => op.symbol === symbol)
  );
}

/**
 * Check if operator is valid for attribute
 */
export function isOperatorValidForAttribute(attributeName: string, operator: string): boolean {
  const attribute = getAttributeByName(attributeName);
  if (!attribute) return true; // Unknown attributes allow any operator
  return attribute.operators.includes(operator);
}

