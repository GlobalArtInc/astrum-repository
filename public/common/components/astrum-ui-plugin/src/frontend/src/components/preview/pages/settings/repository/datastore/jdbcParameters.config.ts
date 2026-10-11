import DataStoreStrings from '../../../../constants/pages/settings/repository/DataStoreStrings';


/**
 * JDBC Parameter Type
 */
export type JdbcParameterType = 'string' | 'number' | 'boolean' | 'enum';

/**
 * JDBC Parameter Definition
 */
export interface JdbcParameterDefinition {
  /** Parameter name (case-sensitive) */
  name: string;
  /** Human-readable description */
  description: string;
  /** Value type */
  type: JdbcParameterType;
  /** Allowed values for enum type */
  allowedValues?: string[];
  /** Default value (informational) */
  defaultValue?: string;
  /** Minimum value for number type */
  min?: number;
  /** Maximum value for number type */
  max?: number;
  /** Unit label (e.g., 'ms', 'seconds') */
  unit?: string;
  /** Category for grouping in UI */
  category: 'connection' | 'timeout' | 'ssl' | 'performance' | 'other';
}

/**
 * Known JDBC Parameters Configuration
 *
 * Astrum's DataStore only supports PostgreSQL and H2. H2 connection settings use an
 * entirely different (SET-style, uppercase) syntax than JDBC Properties, so this catalog
 * is PostgreSQL-only (pgjdbc property names) — see the help panel caption in
 * JdbcParameterEditor.tsx. It's deliberately curated to the parameters admins most
 * commonly need, not the full pgjdbc property list; unknown parameters aren't blocked,
 * just unassisted (no autocomplete/dropdown/validated range).
 *
 * To add a new parameter:
 * 1. Add it to the appropriate category section
 * 2. Specify the type and validation rules
 * 3. Include allowed values for enum types
 */
export const JDBC_PARAMETERS_CONFIG: JdbcParameterDefinition[] = [
  // ===================
  // Connection Parameters
  // ===================
  {
    name: 'ApplicationName',
    description: DataStoreStrings.JDBC_PARAMETERS.APPLICATION_NAME_IDENTIFY_CONNECTION_DATABASE,
    type: 'string',
    category: 'connection',
    defaultValue: 'Astrum Repository',
  },
  {
    name: 'currentSchema',
    description: DataStoreStrings.JDBC_PARAMETERS.DEFAULT_SCHEMA_USE_UNQUALIFIED_TABLE,
    type: 'string',
    category: 'connection',
  },
  {
    name: 'readOnly',
    description: DataStoreStrings.JDBC_PARAMETERS.SET_CONNECTION_READ_ONLY_MODE,
    type: 'boolean',
    allowedValues: ['true', 'false'],
    category: 'connection',
    defaultValue: 'false',
  },

  // ===================
  // Timeout Parameters
  // ===================
  {
    name: 'connectTimeout',
    description: DataStoreStrings.JDBC_PARAMETERS.TIME_WAIT_WHEN_ESTABLISHING_CONNECTION,
    type: 'number',
    min: 0,
    max: 600,
    unit: 'seconds',
    category: 'timeout',
    defaultValue: '10',
  },
  {
    name: 'socketTimeout',
    description: DataStoreStrings.JDBC_PARAMETERS.TIME_WAIT_SOCKET_READ_OPERATIONS,
    type: 'number',
    min: 0,
    max: 600,
    unit: 'seconds',
    category: 'timeout',
    defaultValue: '0',
  },
  {
    name: 'loginTimeout',
    description: DataStoreStrings.JDBC_PARAMETERS.TIME_WAIT_LOGIN_COMPLETE,
    type: 'number',
    min: 0,
    max: 600,
    unit: 'seconds',
    category: 'timeout',
    defaultValue: '0',
  },

  // ===================
  // SSL/TLS Parameters
  // ===================
  {
    name: 'ssl',
    description: DataStoreStrings.JDBC_PARAMETERS.ENABLE_SSL_TLS_ENCRYPTED_CONNECTION,
    type: 'boolean',
    allowedValues: ['true', 'false'],
    category: 'ssl',
    defaultValue: 'false',
  },
  {
    name: 'sslmode',
    description: DataStoreStrings.JDBC_PARAMETERS.SSL_CONNECTION_MODE,
    type: 'enum',
    allowedValues: ['disable', 'allow', 'prefer', 'require', 'verify-ca', 'verify-full'],
    category: 'ssl',
    defaultValue: 'prefer',
  },

  // ===================
  // Performance Parameters
  // ===================
  {
    name: 'reWriteBatchedInserts',
    description: DataStoreStrings.JDBC_PARAMETERS.REWRITE_BATCHED_INSERT_STATEMENTS_BETTER,
    type: 'boolean',
    allowedValues: ['true', 'false'],
    category: 'performance',
    defaultValue: 'false',
  },

  // ===================
  // Other Parameters
  // ===================
  {
    name: 'assumeMinServerVersion',
    description: DataStoreStrings.JDBC_PARAMETERS.ASSUME_MINIMUM_SERVER_VERSION_FEATURE,
    type: 'string',
    category: 'other',
  },
];

/**
 * Get parameter definition by name
 */
export function getParameterDefinition(name: string): JdbcParameterDefinition | undefined {
  return JDBC_PARAMETERS_CONFIG.find(p => p.name.toLowerCase() === name.toLowerCase());
}

/**
 * Get all parameter names for autocomplete
 */
export function getAllParameterNames(): string[] {
  return JDBC_PARAMETERS_CONFIG.map(p => p.name);
}

/**
 * Get parameters by category
 */
export function getParametersByCategory(category: JdbcParameterDefinition['category']): JdbcParameterDefinition[] {
  return JDBC_PARAMETERS_CONFIG.filter(p => p.category === category);
}

/**
 * Get parameter description
 */
export function getParameterDescription(name: string): string {
  const def = getParameterDefinition(name);
  if (!def) return DataStoreStrings.JDBC_PARAMETERS.UNKNOWN_PARAMETER_NOT_STANDARD_JDBC;
  
  let desc = def.description;
  if (def.unit) desc += ` (${unitLabel(def.unit)})`;
  if (def.defaultValue) desc += `. ${DataStoreStrings.JDBC_PARAMETERS.DEFAULT_VALUE(def.defaultValue)}`;
  return desc;
}

/**
 * Check if parameter name is known
 */
export function isKnownParameter(name: string): boolean {
  return JDBC_PARAMETERS_CONFIG.some(p => p.name.toLowerCase() === name.toLowerCase());
}

/**
 * Validate parameter value based on its definition
 */
export function validateParameterValue(name: string, value: string): string | undefined {
  const def = getParameterDefinition(name);
  
  // Unknown parameters - just warn, don't block
  if (!def) {
    return undefined;
  }

  // Empty value is handled separately
  if (!value.trim()) {
    return undefined;
  }

  switch (def.type) {
    case 'boolean':
      if (def.allowedValues && !def.allowedValues.includes(value.toLowerCase())) {
        return DataStoreStrings.JDBC_PARAMETERS.MUST_ONE(def.allowedValues.join(', '));
      }
      break;

    case 'number': {
      const num = parseFloat(value);
      if (Number.isNaN(num)) {
        return DataStoreStrings.JDBC_PARAMETERS.MUST_VALID_NUMBER;
      }
      if (def.min !== undefined && num < def.min) {
        return DataStoreStrings.JDBC_PARAMETERS.MUST_BE_AT_LEAST(`${def.min}${def.unit ? ` ${unitLabel(def.unit)}` : ''}`);
      }
      if (def.max !== undefined && num > def.max) {
        return DataStoreStrings.JDBC_PARAMETERS.MUST_BE_AT_MOST(`${def.max}${def.unit ? ` ${unitLabel(def.unit)}` : ''}`);
      }
      break;
    }

    case 'enum':
      if (def.allowedValues && !def.allowedValues.includes(value)) {
        return DataStoreStrings.JDBC_PARAMETERS.MUST_ONE(def.allowedValues.join(', '));
      }
      break;

    case 'string':
      // No validation for generic strings
      break;
  }

  return undefined;
}

/**
 * Get the display name of a parameter's unit
 */
export function unitLabel(unit: string): string {
  return unit === 'seconds' ? DataStoreStrings.JDBC_PARAMETERS.UNIT_SECONDS : unit;
}

/**
 * Get allowed values for a parameter (for dropdowns)
 */
export function getAllowedValues(name: string): string[] | undefined {
  const def = getParameterDefinition(name);
  return def?.allowedValues;
}

/**
 * Get category display name
 */
export function getCategoryDisplayName(category: JdbcParameterDefinition['category']): string {
  const names: Record<JdbcParameterDefinition['category'], string> = {
    connection: DataStoreStrings.JDBC_PARAMETERS.CONNECTION,
    timeout: DataStoreStrings.JDBC_PARAMETERS.TIMEOUTS,
    ssl: 'SSL/TLS',
    performance: DataStoreStrings.JDBC_PARAMETERS.PERFORMANCE,
    other: DataStoreStrings.JDBC_PARAMETERS.OTHER,
  };
  return names[category];
}


