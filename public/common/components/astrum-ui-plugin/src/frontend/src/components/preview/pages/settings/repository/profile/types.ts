import RepositoryProfileStrings from '../../../../constants/pages/settings/repository/RepositoryProfileStrings';


// =============================================================================
// Repository Profile Types
// Shared types for the Repository Profile page machine, hook, and component
// =============================================================================

// =============================================================================
// Repository Data Types
// =============================================================================

export interface RepositoryProfileData {
  // Basic info
  name: string;
  type: 'hosted' | 'proxy' | 'group';
  format: string;
  url: string;
  online: boolean;
  recipe?: string;

  // Status
  status?: {
    online: boolean;
    description?: string;
    reason?: string;
  };

  // Full configuration attributes
  attributes?: {
    storage?: {
      blobStoreName?: string;
      strictContentTypeValidation?: boolean;
      writePolicy?: string;
    };
    maven?: {
      versionPolicy?: string;
      layoutPolicy?: string;
      contentDisposition?: string;
    };
    proxy?: {
      remoteUrl?: string;
      contentMaxAge?: number;
      metadataMaxAge?: number;
    };
    negativeCache?: {
      enabled?: boolean;
      timeToLive?: number;
    };
    httpClient?: {
      blocked?: boolean;
      autoBlock?: boolean;
      connection?: {
        retries?: number;
        userAgentSuffix?: string;
        timeout?: number;
        enableCircularRedirects?: boolean;
        enableCookies?: boolean;
        useTrustStore?: boolean;
      };
      authentication?: {
        type?: string;
        username?: string;
        ntlmHost?: string;
        ntlmDomain?: string;
      };
    };
    cleanup?: {
      policyNames?: string[];
    };
    component?: {
      proprietaryComponents?: boolean;
    };
    docker?: {
      httpPort?: number;
      httpsPort?: number;
      v1Enabled?: boolean;
      forceBasicAuth?: boolean;
      subdomain?: string;
    };
    dockerProxy?: {
      indexType?: string;
      indexUrl?: string;
      cacheForeignLayers?: boolean;
      foreignLayerUrlWhitelist?: string[];
    };
    group?: {
      memberNames?: string[];
      writableMember?: string;
    };
    replication?: {
      preemptivePullEnabled?: boolean;
      preemptivePullRetryInterval?: number;
    };
    cocoapods?: {
      podBaseUrl?: string;
    };
    conan?: {
      forceBasicAuth?: boolean;
    };
    conda?: {
      assetHistoryWindow?: number;
    };
    go?: {
      loadParentModules?: boolean;
    };
    gitlfs?: {
      strictContentTypeValidation?: boolean;
    };
    helm?: {
      repositoryRootUrl?: string;
    };
    npm?: {
      removeQuarantined?: boolean;
    };
    nugetProxy?: {
      queryCacheItemMaxAge?: number;
      nugetVersion?: string;
    };
    p2?: {
      proxyType?: string;
    };
    r?: {
      pathId?: string;
    };
    rubygems?: {
      storeExtraMetadata?: boolean;
    };
    yum?: {
      repodataDepth?: number;
      groupId?: string;
    };
    apt?: {
      distribution?: string;
      flat?: boolean;
    };
    aptSigning?: {
      keyId?: string;
      passphrase?: string;
    };
    raw?: {
      contentDisposition?: string;
    };
  };
  routingRuleId?: string;
}

// =============================================================================
// IQ / Firewall Types
// =============================================================================

/**
 * Health check data for a repository.
 * Fields match HealthCheckRepositoryStatusXO from the backend.
 */
export interface HealthCheckData {
  enabled: boolean;
  analyzing?: boolean;
  securityIssueCount?: number;
  licenseIssueCount?: number;
  malwareCount?: number | null;
  vulnerableCounts?: number[];
  totalCounts?: number[];
  summaryUrl?: string;
  detailUrl?: string;
  lastAnalyzedDate?: number;
}

export interface FirewallData {
  enabled: boolean;
  quarantineEnabled?: boolean;
  affectedComponentCount?: number;
  criticalComponentCount?: number;
  severeComponentCount?: number;
  moderateComponentCount?: number;
  quarantinedComponentCount?: number;
  reportUrl?: string;
  message?: string;
  errorMessage?: string;
}

export interface MalwareCleanupSummary {
  repositoryName: string;
  scrubbedCount: number;
  pendingCount: number;
  lastRun?: string;
  taskStatus: string;
  taskEnabled: boolean;
  /** Whether the task has enableMalwareCleanup=true (delete mode vs audit-only) */
  taskCleanupEnabled: boolean;
}

export interface IqApplicationMapping {
  applicationId: string;
  applicationName?: string;
  repositoryName: string;
}

export interface IqCapabilities {
  connected: boolean;
  hasFirewall: boolean;
  hasLifecycle: boolean;
  url?: string;
  deploymentId?: string;
}

// =============================================================================
// Metrics Types
// =============================================================================

export interface RepositoryMetrics {
  componentCount: number;
  assetCount: number;
  totalSize: number;
  downloadsPerMonth: number;
  uploadsPerMonth: number;
}

// =============================================================================
// Blob Store Types
// =============================================================================

export interface BlobStoreInfo {
  name: string;
  type: string;
  typeId?: string;
  path?: string;
  unavailable: boolean;
  totalSizeInBytes?: number;
  availableSpaceInBytes?: number;
  blobCount?: number;
}

export interface CleanupPolicyInfo {
  name: string;
  format: string;
  notes?: string;
  lastRun?: string;
}

export interface RoutingRuleInfo {
  name: string;
  description?: string;
  mode: string;
  matchers: string[];
}

// =============================================================================
// Security Types
// =============================================================================

export interface PrivilegeInfo {
  name: string;
  description?: string;
  type: string;
  repository?: string;
  format?: string;
  actions?: string[];
}

export interface RoleInfo {
  id: string;
  name: string;
  description?: string;
  privileges: string[];
  roles: string[];
}

export interface UserWithAccess {
  userId: string;
  firstName?: string;
  lastName?: string;
  email?: string;
  roles?: string[];
  permissions?: string[];
}

export interface AnonymousAccess {
  enabled: boolean;
  userId?: string;
  realmName?: string;
  realm?: string;
  canRead?: boolean;
  canBrowse?: boolean;
}

// =============================================================================
// System Types
// =============================================================================

export interface TaskInfo {
  id: string;
  name: string;
  type: string;
  schedule?: string;
  lastRun?: string;
  lastRunResult?: string;
  nextRun?: string;
  currentState?: string;
}

export interface CapabilityInfo {
  id: string;
  type: string;
  enabled: boolean;
  notes?: string;
  affectsRepo?: boolean;
  affectsReason?: string;
  properties?: Record<string, string>;
}

export interface HttpSettingsInfo {
  userAgentSuffix?: string;
  httpProxy?: {
    host: string;
    port: number;
  };
  httpsProxy?: {
    host: string;
    port: number;
  };
  nonProxyHosts?: string[];
}

// =============================================================================
// Action Types (for machine)
// =============================================================================

/**
 * Actions that require confirmation before execution.
 */
export type RepositoryAction =
  | 'invalidate-cache'
  | 'rebuild-index'
  | 'toggle-online';

/**
 * Action metadata for confirmation dialogs.
 */
export interface ActionMetadata {
  title: string;
  message: string;
  confirmLabel: string;
  variant: 'warning' | 'danger';
}

/**
 * Map of action types to their confirmation dialog content.
 */
export const ACTION_METADATA: Record<RepositoryAction, ActionMetadata> = {
  'invalidate-cache': {
    title: RepositoryProfileStrings.TYPES.INVALIDATE_CACHE,
    message: RepositoryProfileStrings.TYPES.CLEAR_ALL_CACHED_CONTENT_PROXY,
    confirmLabel: RepositoryProfileStrings.TYPES.INVALIDATE_CACHE,
    variant: 'warning',
  },
  'rebuild-index': {
    title: RepositoryProfileStrings.TYPES.REBUILD_INDEX,
    message: RepositoryProfileStrings.TYPES.REBUILD_SEARCH_INDEX_REPOSITORY_PROCESS,
    confirmLabel: RepositoryProfileStrings.TYPES.REBUILD_INDEX,
    variant: 'warning',
  },
  'toggle-online': {
    title: RepositoryProfileStrings.TYPES.TOGGLE_REPOSITORY_STATUS,
    message: RepositoryProfileStrings.TYPES.CHANGE_ONLINE_STATUS_REPOSITORY_USERS,
    confirmLabel: RepositoryProfileStrings.TYPES.CONTINUE,
    variant: 'warning',
  },
};

// =============================================================================
// Tab Types
// =============================================================================

export type TabId =
  | 'repository'
  | 'structure'
  | 'membership'
  | 'usage'
  | 'audit'
  | 'security'
  | 'system'
  | 'instance-config';

export const TABS: Array<{ id: TabId; label: string }> = [
  { id: 'repository', label: RepositoryProfileStrings.TYPES.REPOSITORY },
  { id: 'structure', label: RepositoryProfileStrings.TYPES.STRUCTURE },
  { id: 'membership', label: RepositoryProfileStrings.TYPES.GROUP_MEMBERSHIP },
  { id: 'usage', label: RepositoryProfileStrings.TYPES.USAGE },
  { id: 'audit', label: RepositoryProfileStrings.TYPES.AUDIT },
  { id: 'security', label: RepositoryProfileStrings.TYPES.ACCESS_SECURITY },
  { id: 'system', label: RepositoryProfileStrings.TYPES.SYSTEM },
  { id: 'instance-config', label: RepositoryProfileStrings.TYPES.INSTANCE_CONFIG },
];
