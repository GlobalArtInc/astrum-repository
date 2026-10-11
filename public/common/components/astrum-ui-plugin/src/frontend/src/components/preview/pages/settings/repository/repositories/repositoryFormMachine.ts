

import { assign } from 'xstate';
import { API_INTERNAL_UI, ENDPOINTS, restClient } from '../../../../../../interface/api';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';

import {
  Repository,
  RepositoryFormData,
  RepositoryType,
  BlobStore,
  RoutingRule,
  CleanupPolicy,
  RepositoryReference,
  Recipe,
  NAME_PATTERN,
  NAME_PATTERN_MESSAGE,
  DEFAULT_HOSTED_VALUES,
  DEFAULT_PROXY_VALUES,
  DEFAULT_GROUP_VALUES,
} from './types';
import CommonStrings from '../../../../constants/CommonStrings';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import UIStrings from '../../../../../../constants/pages/admin/repository/RepositoriesStrings';

// =============================================================================
// Constants
// =============================================================================

/** Repository type constants for machine sub-states */
export const REPOSITORY_TYPES = {
  HOSTED: 'hosted' as const,
  PROXY: 'proxy' as const,
  GROUP: 'group' as const,
};

/** Validation error key for the Docker proxy foreign-layer URL whitelist field. */
export const DOCKER_FOREIGN_LAYER_WHITELIST_ERROR_KEY = 'dockerProxy.foreignLayerUrlWhitelist';

// Internal UI endpoints for reference data
const RECIPES_URL = `${API_INTERNAL_UI}/repositories/recipes`;
const REPOSITORIES_LIST_URL = `${API_INTERNAL_UI}/repositories`;
const CLEANUP_POLICIES_URL = '/service/rest/internal/cleanup-policies';

// =============================================================================
// Guards
// =============================================================================

/**
 * Guard factory: creates a guard that checks if a TYPE_CHANGE event targets a specific type.
 */
const isTypeGuard = (targetType: string) =>
  (_context: unknown, event: { type: string; value?: string }) => event.value === targetType;

// =============================================================================
// Validation
// =============================================================================

/**
 * Returns true when an entry looks like a regex pattern. Used by the Docker
 * proxy foreign-layer whitelist validator to bypass URL-format validation for
 * legacy regex entries (e.g. Classic UI's ".*" default seed, anchored patterns
 * like "^https://.*$").
 *
 * Two-step gate to avoid letting URL typos slip through as "valid regex":
 *   1. Entries that look like a URL ("http://..." / "https://...") always go
 *      to URL validation, even if they contain regex meta-chars.
 *   2. Otherwise, require a strong regex signal (anchors, character classes,
 *      alternation, brackets, or escape sequences). The trigger set excludes
 *      "?" and "+" because they are common in URL query strings and would
 *      let typos like "example.com?path" silently bypass validation.
 *
 * Java/JS regex engines have minor syntax differences (possessive quantifiers,
 * named groups). A pattern accepted by Java but rejected by JS would be flagged
 * client-side as "Invalid URL format"; the server's Pattern.compile() is the
 * authoritative validator for anything that gets past this gate.
 */
function looksLikeRegexPattern(entry: string): boolean {
  if (/^https?:\/\//i.test(entry)) return false;
  if (!/[*^$|[\]{}]|\\[dDsSwWbB]/.test(entry)) return false;
  try {
    new RegExp(entry);
    return true;
  } catch {
    return false;
  }
}

/**
 * Validate repository form data.
 * Returns flat validation errors using dot notation for nested fields,
 * compatible with the createFormMachine validation contract.
 */
export function validateRepository(data: RepositoryFormData, options?: { isCloud?: boolean }): ValidationErrors {
  const errors: ValidationErrors = {};

  // Name is always required
  if (!data.name?.trim()) {
    errors.name = CommonStrings.MESSAGES.NAME_REQUIRED;
  } else if (!NAME_PATTERN.test(data.name)) {
    errors.name = NAME_PATTERN_MESSAGE;
  }

  // Blob store is required in self-hosted only (cloud manages its own storage)
  if (!options?.isCloud && !data.storage?.blobStoreName?.trim()) {
    errors['storage.blobStoreName'] = RepositoriesPageStrings.FORM_MACHINE.BLOB_STORE_REQUIRED;
  }

  // Proxy-specific validation
  if (data.type === 'proxy') {
    if (!data.proxy?.remoteUrl?.trim()) {
      errors['proxy.remoteUrl'] = RepositoriesPageStrings.FORM_MACHINE.REMOTE_URL_REQUIRED;
    } else {
      try {
        new URL(data.proxy.remoteUrl);
      } catch {
        errors['proxy.remoteUrl'] = RepositoriesPageStrings.FORM_MACHINE.INVALID_URL_FORMAT;
      }
    }

    // Proxy numeric fields validation (contentMaxAge, metadataMaxAge)
    // These fields are optional but must be valid numbers when provided
    const contentMaxAge = data.proxy?.contentMaxAge;
    if (contentMaxAge != null && (Number.isNaN(contentMaxAge) || contentMaxAge < -1)) {
      errors['proxy.contentMaxAge'] = RepositoriesPageStrings.FORM_MACHINE.MAXIMUM_COMPONENT_AGE_MUST_VALID;
    }

    const metadataMaxAge = data.proxy?.metadataMaxAge;
    // Match Classic UI's validateTimeToLive: allow any value >= -1 (including 0).
    // -1 means "cache forever" — a legitimate, supported value. Tightening this
    // to >0 blocked editing existing Maven2 proxy repos that had been configured
    // with -1, manifesting as a silent Save failure.
    if (metadataMaxAge != null && (Number.isNaN(metadataMaxAge) || metadataMaxAge < -1)) {
      errors['proxy.metadataMaxAge'] = RepositoriesPageStrings.FORM_MACHINE.MAXIMUM_METADATA_AGE_MUST_VALID;
    }

    // Negative cache validation — validate TTL regardless of enabled state so
    // an invalid value saved while disabled is caught before re-enabling trips it.
    const timeToLive = data.negativeCache?.timeToLive;
    if (timeToLive != null && (Number.isNaN(timeToLive) || timeToLive < 0)) {
      errors['negativeCache.timeToLive'] = RepositoriesPageStrings.FORM_MACHINE.TTL_MUST_0_GREATER;
    }
    if (data.negativeCache?.enabled && (timeToLive == null || Number.isNaN(timeToLive) || timeToLive < 0)) {
      errors['negativeCache.timeToLive'] = RepositoriesPageStrings.FORM_MACHINE.TTL_REQUIRED_WHEN_NEGATIVE_CACHE;
    }

    // HTTP Authentication validation — fields required vary by auth type
    const authType = data.httpClient?.authentication?.type;
    if (authType === 'bearer') {
      if (!data.httpClient?.authentication?.bearerToken?.trim()) {
        errors['httpClient.authentication.bearerToken'] = RepositoriesPageStrings.FORM_MACHINE.BEARER_TOKEN_REQUIRED;
      }
    } else if (authType) {
      // username/ntlm auth types all require username + password
      if (!data.httpClient?.authentication?.username?.trim()) {
        errors['httpClient.authentication.username'] = RepositoriesPageStrings.FORM_MACHINE.USERNAME_REQUIRED_WHEN_AUTHENTICATION_ENABLED;
      }
      if (!data.httpClient?.authentication?.password?.trim()) {
        errors['httpClient.authentication.password'] = RepositoriesPageStrings.FORM_MACHINE.PASSWORD_REQUIRED_WHEN_AUTHENTICATION_ENABLED;
      }
      // NTLM-specific fields
      if (authType === 'ntlm') {
        if (!data.httpClient?.authentication?.ntlmHost?.trim()) {
          errors['httpClient.authentication.ntlmHost'] = RepositoriesPageStrings.FORM_MACHINE.NTLM_HOST_REQUIRED_NTLM_AUTHENTICATION;
        }
        if (!data.httpClient?.authentication?.ntlmDomain?.trim()) {
          errors['httpClient.authentication.ntlmDomain'] = RepositoriesPageStrings.FORM_MACHINE.NTLM_DOMAIN_REQUIRED_NTLM_AUTHENTICATION;
        }
      }
    }
  }

  // Group-specific validation
  if (data.type === 'group') {
    if (!data.group?.memberNames || data.group.memberNames.length === 0) {
      errors['group.memberNames'] = RepositoriesPageStrings.FORM_MACHINE.LEAST_ONE_MEMBER_REPOSITORY_REQUIRED;
    }
  }

  // Format-specific validation
  if (data.format === 'apt' && data.type === 'hosted') {
    if (!data.aptSigning?.keypair?.trim()) {
      errors['aptSigning.keypair'] = RepositoriesPageStrings.FORM_MACHINE.GPG_SIGNING_KEY_REQUIRED_APT;
    }
  }

  if (data.format === 'alpine' && data.type === 'group') {
    if (!data.alpineSigning?.keypair?.trim()) {
      errors['alpineSigning.keypair'] = RepositoriesPageStrings.FORM_MACHINE.RSA_SIGNING_KEY_REQUIRED_ALPINE;
    }
  }

  // Docker port validation
  if (data.format === 'docker' && !data.docker?.pathEnabled) {
    const httpPort = data.docker?.httpPort;
    const httpsPort = data.docker?.httpsPort;
    if (httpPort != null && (httpPort < 1 || httpPort > 65535)) {
      errors['docker.httpPort'] = RepositoriesPageStrings.FORM_MACHINE.PORT_MUST_BETWEEN_1_65535;
    }
    if (httpsPort != null && (httpsPort < 1 || httpsPort > 65535)) {
      errors['docker.httpsPort'] = RepositoriesPageStrings.FORM_MACHINE.PORT_MUST_BETWEEN_1_65535;
    }
  }

  // Docker proxy foreign-layer URL whitelist validation: each entry must parse
  // as a URL on the client. Entries that look like regex patterns and compile
  // as such are allowed through, preserving backward compatibility with the
  // Classic UI's default seed value of ".*" and any user-entered regex patterns
  // the server's Pattern.compile() accepts.
  if (data.format === 'docker' && data.type === 'proxy' && data.dockerProxy?.cacheForeignLayers) {
    for (const raw of data.dockerProxy.foreignLayerUrlWhitelist ?? []) {
      const entry = raw.trim();
      if (!entry) continue;
      if (looksLikeRegexPattern(entry)) continue;
      try {
        new URL(entry);
      } catch {
        errors[DOCKER_FOREIGN_LAYER_WHITELIST_ERROR_KEY] = RepositoriesPageStrings.FORM_MACHINE.INVALID_URL_FORMAT_2(entry);
        break;
      }
    }
  }

  // Docker proxy custom index URL validation.
  // Mirrors the backend @Url constraint (DockerProxyFacetSupport.Config#indexUrl)
  // for immediate UX feedback; the backend remains the source of truth.
  if (data.format === 'docker' && data.type === 'proxy' && data.dockerProxy?.indexType === 'CUSTOM') {
    const indexUrl = data.dockerProxy.indexUrl?.trim();
    if (!indexUrl) {
      errors['dockerProxy.indexUrl'] = RepositoriesPageStrings.FORM_MACHINE.INDEX_URL_REQUIRED;
    } else {
      try {
        const parsed = new URL(indexUrl);
        if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
          errors['dockerProxy.indexUrl'] = RepositoriesPageStrings.FORM_MACHINE.INDEX_URL_MUST_USE_HTTP;
        }
      } catch {
        errors['dockerProxy.indexUrl'] = RepositoriesPageStrings.FORM_MACHINE.INVALID_URL_FORMAT;
      }
    }
  }

  // Terraform signing: a passphrase without a key cannot sign anything.
  if (data.format === 'terraform' && data.type === 'hosted') {
    if (data.terraformSigning?.passphrase?.trim() && !data.terraformSigning?.keypair?.trim()) {
      errors['terraformSigning.keypair'] = RepositoriesPageStrings.FORM_MACHINE.SIGNING_KEY_REQUIRED_WITH_PASSPHRASE;
    }
  }

  // OCI connector ports, cosign keyless policy and custom proxy index
  if (data.format === 'oci') {
    const httpPort = data.oci?.httpPort;
    const httpsPort = data.oci?.httpsPort;
    if (httpPort != null && (Number.isNaN(httpPort) || httpPort < 1 || httpPort > 65535)) {
      errors['oci.httpPort'] = RepositoriesPageStrings.FORM_MACHINE.PORT_MUST_BETWEEN_1_65535;
    }
    if (httpsPort != null && (Number.isNaN(httpsPort) || httpsPort < 1 || httpsPort > 65535)) {
      errors['oci.httpsPort'] = RepositoriesPageStrings.FORM_MACHINE.PORT_MUST_BETWEEN_1_65535;
    }
    if (data.oci?.cosign?.enforcement === 'KEYLESS') {
      if (!data.oci.cosign.identityRegex?.trim()) {
        errors['oci.cosign.identityRegex'] = UIStrings.OCI.IDENTITY_REGEX_REQUIRED_WHEN_KEYLESS;
      }
      if (!data.oci.cosign.issuerRegex?.trim()) {
        errors['oci.cosign.issuerRegex'] = UIStrings.OCI.ISSUER_REGEX_REQUIRED_WHEN_KEYLESS;
      }
    }
    if (data.type === 'proxy' && data.oci?.ociProxy?.indexType === 'CUSTOM') {
      const indexUrl = data.oci.ociProxy.indexUrl?.trim();
      if (!indexUrl) {
        errors['oci.ociProxy.indexUrl'] = UIStrings.OCI.INDEX_URL_REQUIRED_WHEN_CUSTOM;
      } else {
        try {
          new URL(indexUrl);
        } catch {
          errors['oci.ociProxy.indexUrl'] = RepositoriesPageStrings.FORM_MACHINE.INVALID_URL_FORMAT;
        }
      }
    }
  }

  // NuGet proxy validation
  if (data.format === 'nuget' && data.type === 'proxy') {
    const queryCacheItemMaxAge = data.nugetProxy?.queryCacheItemMaxAge;
    if (queryCacheItemMaxAge != null && (Number.isNaN(queryCacheItemMaxAge) || queryCacheItemMaxAge < 0)) {
      errors['nugetProxy.queryCacheItemMaxAge'] = RepositoriesPageStrings.FORM_MACHINE.CACHE_AGE_MUST_0_GREATER;
    }
  }

  return errors;
}

// =============================================================================
// API Helpers
// =============================================================================

/**
 * Fetch a single repository by name from the REST API
 */
async function findRepository(name: string): Promise<Repository | null> {
  try {
    const data = await restClient.get<Repository>(
      `${ENDPOINTS.REPOSITORIES}/${encodeURIComponent(name)}`
    );
    return data || null;
  } catch (err) {
    console.error('Failed to load repository:', err);
    throw err;
  }
}

/**
 * Build form data from a loaded repository (handles both flat and nested API response formats)
 */
function buildFormDataFromRepository(repo: Repository): RepositoryFormData {
  const repoData = repo as Repository & Record<string, unknown>;

  const getConfig = <T>(key: string): T | undefined => {
    return (repoData[key] as T) ?? (repo.attributes as Record<string, T> | undefined)?.[key];
  };

  const storageConfig = getConfig<{
    blobStoreName?: string;
    strictContentTypeValidation?: boolean;
    writePolicy?: string;
  }>('storage');

  const blobStoreName =
    storageConfig?.blobStoreName ||
    (repoData.blobStoreName as string | undefined) ||
    repo.attributes?.storage?.blobStoreName ||
    '';

  return {
    name: repo.name,
    type: repo.type,
    format: repo.format,
    recipe: repo.recipe || `${repo.format}-${repo.type}`,
    online: repo.online ?? true,
    routingRuleId: repo.routingRuleId ?? (repoData.routingRuleName as string | null | undefined),
    storage: {
      blobStoreName,
      strictContentTypeValidation: storageConfig?.strictContentTypeValidation ?? true,
      writePolicy: storageConfig?.writePolicy as 'ALLOW' | 'ALLOW_ONCE' | 'DENY' | undefined,
    },
    proxy: getConfig('proxy'),
    negativeCache: getConfig('negativeCache'),
    httpClient: getConfig('httpClient'),
    group: getConfig('group'),
    cleanup: getConfig('cleanup'),
    component: getConfig('component'),
    replication: getConfig('replication'),
    maven: getConfig('maven'),
    docker: getConfig('docker'),
    dockerProxy: getConfig('dockerProxy'),
    nugetProxy: getConfig('nugetProxy'),
    apt: getConfig('apt'),
    aptSigning: getConfig('aptSigning'),
    alpineSigning: getConfig('alpineSigning'),
    yum: getConfig('yum'),
    yumSigning: getConfig('yumSigning'),
    raw: getConfig('raw'),
    // Without this, an existing PyPI proxy's saved `indexPath` would not seed
    // the form on edit, and a save without touching that section would omit
    // `pypi` from the PUT body entirely (silent no-op for the user).
    // (Post-migration STL-381: `removeQuarantinedVersions` was removed; PCCS
    // is now expressed via `firewall.mode = "PCCS"` on the top-level config.)
    pypi: getConfig('pypi'),
    terraformSigning: withoutPassphrase(getConfig<RepositoryFormData['terraformSigning']>('terraformSigning')),
    cargo: getConfig('cargo'),
    conanProxy: getConfig('conanProxy'),
    oci: getConfig('oci'),
  };
}

/**
 * The Terraform signing passphrase is write-only: whatever the server returns for it never reaches the
 * form, so it cannot be shown or sent back unchanged.
 */
function withoutPassphrase(
  signing: RepositoryFormData['terraformSigning'] | undefined
): RepositoryFormData['terraformSigning'] | undefined {
  if (!signing) {
    return signing;
  }
  const { passphrase: _passphrase, ...rest } = signing;
  return rest as RepositoryFormData['terraformSigning'];
}

/**
 * Build default form data for a new repository based on format and type
 */
function buildDefaultFormData(format: string, type: RepositoryType): RepositoryFormData {
  const baseValues: RepositoryFormData = {
    name: '',
    type,
    format,
    recipe: `${format}-${type}`,
    online: true,
    storage: {
      blobStoreName: '',
      strictContentTypeValidation: true,
    },
  };

  // Format-specific defaults
  // Maven2 (hosted/proxy) defaults: 'Attachment' content disposition (matches classic UI / API)
  if (format === 'maven2' && type !== 'group') {
    baseValues.maven = {
      versionPolicy: 'RELEASE',
      layoutPolicy: 'STRICT',
      contentDisposition: 'ATTACHMENT',
    };
  }

  // NuGet proxy default: Query Cache Item Max Age = 3600 seconds (matches classic UI / API).
  // Without this seed, the field renders blank on create and the API serializer
  // applies the default at submit-time, hiding the value from the user.
  if (format === 'nuget' && type === 'proxy') {
    baseValues.nugetProxy = {
      queryCacheItemMaxAge: 3600,
      nugetVersion: 'V3',
      symbolServerUrl: '',
      allowAnonymousSymbolAccess: true,
    };
  }

  // PyPI proxy defaults. Seeding `pypi` here ensures the create payload always
  // carries a `pypi` block so the backend converter runs (it skips the block
  // entirely when the request field is null, leaving the repo without any
  // pypi attributes set). Default `indexPath` matches the Classic UI and the
  // backend's own `/simple` fallback.
  // (Post-migration STL-381: `removeQuarantinedVersions` was removed from
  // PypiConfig — PCCS is now expressed via `firewall.mode = "PCCS"`.)
  if (format === 'pypi' && type === 'proxy') {
    baseValues.pypi = {
      indexPath: '/simple',
    };
  }

  // Cargo: anonymous reads follow the instance setting until the admin requires authentication.
  if (format === 'cargo') {
    baseValues.cargo = { requireAuthentication: false };
  }

  // Conan proxy: new proxies speak Conan 2 unless the admin picks Conan 1 (matches the Classic UI).
  if (format === 'conan' && type === 'proxy') {
    baseValues.conanProxy = { conanVersion: 'V2' };
  }

  // OCI: path-based routing and no cosign enforcement, as the OCI facet shows by default.
  if (format === 'oci') {
    baseValues.oci = {
      pathEnabled: true,
      forceBasicAuth: false,
      cosign: { enforcement: 'NONE' },
      ...(type === 'proxy'
        ? { ociProxy: { indexType: 'REGISTRY', cacheForeignLayers: false, foreignLayerUrlWhitelist: [] } }
        : {}),
    };
  }

  // npm proxy: post-migration STL-381 there is no longer any npm-specific
  // form data to seed. The legacy `removeQuarantinedVersions` flag was npm's
  // only field and has been removed; `NpmConfig` no longer exists in types.ts.
  // PCCS is now expressed via `firewall.mode = "PCCS"` on the top-level
  // repository config (set via the Firewall tab).

  switch (type) {
    case 'hosted':
      return { ...baseValues, ...DEFAULT_HOSTED_VALUES } as RepositoryFormData;
    case 'proxy':
      return { ...baseValues, ...DEFAULT_PROXY_VALUES } as RepositoryFormData;
    case 'group':
      return { ...baseValues, ...DEFAULT_GROUP_VALUES } as RepositoryFormData;
    default:
      return baseValues;
  }
}

// =============================================================================
// Machine Factory
// =============================================================================

export interface RepositoryFormMachineOptions {
  /** Repository name for edit mode; undefined for create mode */
  repositoryName?: string;
  /** Pre-loaded repository to avoid re-fetching */
  preloadedRepository?: Repository;
  /** Format for the repository (e.g., 'maven2', 'npm', 'docker') */
  format: string;
  /** Repository type for create mode (defaults to 'hosted') */
  repositoryType?: RepositoryType;
}

/**
 * Create a repository form machine with XState.
 *
 * The machine models repository types (hosted/proxy/group) as sub-states
 * within the editing state. Each sub-state declares field metadata for
 * the UI component to consume without switch/case logic.
 *
 * Format is a context field that affects validation, not a sub-state.
 * Format-specific fields (maven, docker, apt, etc.) are rendered based
 * on the format value in context.data.format.
 *
 * Supports TYPE_CHANGE transitions between hosted/proxy/group, which:
 * 1. Transition to the correct sub-state
 * 2. Reset type-specific fields
 * 3. Apply default values for the new type
 * 4. Preserve common fields (name, format, blobStoreName)
 */
export function createRepositoryFormMachine(options: RepositoryFormMachineOptions) {
  const { repositoryName, preloadedRepository, format, repositoryType = 'hosted' } = options;

  const initialType: RepositoryType = preloadedRepository?.type ?? repositoryType;
  const initialData = preloadedRepository
    ? buildFormDataFromRepository(preloadedRepository)
    : buildDefaultFormData(format, initialType);

  return createFormMachine({
    id: `repository-form-${repositoryName ?? 'new'}`,
    context: {
      data: initialData,
      // Reference data - populated by the load service
      repository: preloadedRepository ?? (null as Repository | null),
      blobStores: [] as BlobStore[],
      routingRules: [] as RoutingRule[],
      cleanupPolicies: [] as CleanupPolicy[],
      memberRepositories: [] as RepositoryReference[],
      recipes: [] as Recipe[],
    },
    actions: {
      validate: assign((ctx: FormContext<RepositoryFormData>) => ({
        validationErrors: validateRepository(ctx.data, {
          isCloud: ExtJS.state()?.getValue('isCloud', false) ?? false,
        }),
      })),
      // Custom action: update type, reset type-specific fields, apply defaults.
      // Preserves common fields (name, format, online, blobStoreName).
      changeType: assign((context: any, event: any) => {
        const newType = event.value as RepositoryType;
        const currentData = context.data as RepositoryFormData;

        // Preserve fields common to all types
        const preserved = {
          name: currentData.name,
          format: currentData.format,
          online: currentData.online,
        };

        // Get default values for the new type
        let typeDefaults: Partial<RepositoryFormData>;
        switch (newType) {
          case 'hosted':
            typeDefaults = { ...DEFAULT_HOSTED_VALUES };
            break;
          case 'proxy':
            typeDefaults = { ...DEFAULT_PROXY_VALUES };
            break;
          case 'group':
            typeDefaults = { ...DEFAULT_GROUP_VALUES };
            break;
          default:
            typeDefaults = {};
        }

        // Format-specific defaults that the type-only DEFAULT_* maps don't
        // carry. These mirror the logic in buildDefaultFormData so that a
        // TYPE_CHANGE landing on (e.g.) pypi-proxy seeds the same self-
        // describing payload that a fresh pypi-proxy create would. Without
        // these, switching TO pypi-proxy via TYPE_CHANGE would leave
        // data.pypi undefined and the form would render the firewall
        // checkbox unchecked even though buildRepositoryConfig now defaults
        // it sensibly on save.
        const formatDefaults: Partial<RepositoryFormData> = {};
        if (currentData.format === 'nuget' && newType === 'proxy') {
          formatDefaults.nugetProxy = {
            queryCacheItemMaxAge: 3600,
            nugetVersion: 'V3',
            symbolServerUrl: '',
            allowAnonymousSymbolAccess: true,
          };
        }
        if (currentData.format === 'pypi' && newType === 'proxy') {
          // `removeQuarantinedVersions` removed post-migration STL-381 — see PypiConfig in types.ts.
          formatDefaults.pypi = { indexPath: '/simple' };
        }
        // npm proxy has no format-specific defaults post-migration STL-381 (NpmConfig was removed).

        // Build new data: type defaults + format defaults + preserved common fields + explicit overrides
        const newData: RepositoryFormData = {
          ...typeDefaults,
          ...formatDefaults,
          ...preserved,
          type: newType,
          recipe: `${currentData.format}-${newType}`,
          storage: {
            ...(typeDefaults.storage || { blobStoreName: '', strictContentTypeValidation: true }),
            blobStoreName: currentData.storage?.blobStoreName || '',
          },
        } as RepositoryFormData;

        return {
          data: newData,
          touched: { ...context.touched, type: true },
        };
      }),
    },
    // Guards for TYPE_CHANGE transitions between sub-states
    guards: {
      isTypeHosted: isTypeGuard(REPOSITORY_TYPES.HOSTED) as any,
      isTypeProxy: isTypeGuard(REPOSITORY_TYPES.PROXY) as any,
      isTypeGroup: isTypeGuard(REPOSITORY_TYPES.GROUP) as any,
    },
    services: {
      load: async () => {
        // Load repository and reference data in parallel
        const [repository, blobStoresData, routingRulesData, cleanupData, recipesData, memberData] =
          await Promise.all([
            // Load repository if editing (use preloaded if available)
            preloadedRepository
              ? Promise.resolve(preloadedRepository)
              : repositoryName
                ? findRepository(repositoryName).catch((err: unknown) => {
                    console.error('Failed to load repository:', err);
                    throw err;
                  })
                : Promise.resolve(null),
            // Load blob stores
            (ExtJS.state()?.getValue('isCloud', false)
              ? Promise.resolve([] as BlobStore[])
              : restClient
                  .get(ENDPOINTS.BLOBSTORES)
                  .then((data: unknown) => data as BlobStore[])
                  .catch((err: unknown) => {
                    console.warn('Could not load blob stores:', err);
                    return [] as BlobStore[];
                  })),
            // Load routing rules
            restClient
              .get(ENDPOINTS.ROUTING_RULES)
              .then((data: unknown) => data as RoutingRule[])
              .catch((err: unknown) => {
                console.warn('Could not load routing rules:', err);
                return [] as RoutingRule[];
              }),
            // Load cleanup policies
            restClient
              .get(CLEANUP_POLICIES_URL)
              .then((data: unknown) => data as CleanupPolicy[])
              .catch((err: unknown) => {
                console.warn('Could not load cleanup policies:', err);
                return [] as CleanupPolicy[];
              }),
            // Load recipes
            restClient
              .get(RECIPES_URL)
              .then((data: unknown) => {
                const rawRecipes = data as Array<{ format: string; type: string }>;
                return Array.isArray(rawRecipes)
                  ? rawRecipes.map((r) => ({ ...r, name: `${r.format}-${r.type}` }))
                  : [];
              })
              .catch((err: unknown) => {
                console.warn('Could not load recipes:', err);
                return [] as Recipe[];
              }),
            // Load member repositories — use /details to get type and format fields,
            // then filter client-side by format (group repos can only contain same-format repos)
            restClient
              .get(`${REPOSITORIES_LIST_URL}/details`)
              .then((data: unknown) => {
                const repos = data as Array<{ name: string; format: string; type: string }>;
                return Array.isArray(repos)
                  ? repos
                      .filter((r) => r.format?.toLowerCase() === format.toLowerCase())
                      .map((r) => ({
                        id: r.name,
                        name: r.name,
                        format: r.format,
                        type: r.type as RepositoryType,
                      }))
                  : [];
              })
              .catch((err: unknown) => {
                console.warn('Could not load repository references:', err);
                return [] as RepositoryReference[];
              }),
          ]);

        // Process reference data
        const blobStores = Array.isArray(blobStoresData) ? blobStoresData : [];
        const routingRules = Array.isArray(routingRulesData) ? routingRulesData : [];
        const cleanupPolicies = Array.isArray(cleanupData) ? cleanupData : [];
        const recipes = Array.isArray(recipesData) ? recipesData : [];
        const memberRepositories = Array.isArray(memberData)
          ? memberData.filter((r) => r.name !== repositoryName && r.type !== 'group')
          : [];

        // Build form data from loaded repository or use defaults
        const formData: RepositoryFormData = repository
          ? buildFormDataFromRepository(repository as Repository)
          : buildDefaultFormData(format, initialType);

        // Auto-select the sole blob store when creating a new repository
        if (!repositoryName && blobStores.length === 1 && !formData.storage?.blobStoreName) {
          formData.storage = {
            ...(formData.storage || { strictContentTypeValidation: true }),
            blobStoreName: blobStores[0].name,
          };
        }

        return {
          data: formData,
          repository: repository as Repository | null,
          blobStores,
          routingRules,
          cleanupPolicies,
          recipes,
          memberRepositories,
        };
      },
      // save service is provided via useForm options
    },
    // Custom event for repository type changes (transitions to correct sub-state)
    on: {
      TYPE_CHANGE: [
        {
          target: '.hosted',
          cond: 'isTypeHosted',
          actions: ['changeType', 'validate', 'computePristine'],
        },
        {
          target: '.proxy',
          cond: 'isTypeProxy',
          actions: ['changeType', 'validate', 'computePristine'],
        },
        {
          target: '.group',
          cond: 'isTypeGroup',
          actions: ['changeType', 'validate', 'computePristine'],
        },
      ],
    },
    // Repository type variant sub-states within the editing state.
    // Each sub-state declares metadata about which fields are visible and required
    // for that type variant. This enables:
    // 1. The component to read field config from machine state (no switch/case)
    // 2. Model-based testing that auto-generates paths through every variant
    // 3. Single source of truth for form structure per repository type
    editingConfig: {
      defaultState: initialType,
      typeField: 'type',
      states: {
        [REPOSITORY_TYPES.HOSTED]: {
          meta: {
            typeLabel: RepositoriesPageStrings.FORM_MACHINE.HOSTED,
            fields: [
              'storage.blobStoreName',
              'storage.strictContentTypeValidation',
              'storage.writePolicy',
              'component.proprietaryComponents',
              'cleanup.policyNames',
            ],
            requiredFields: ['storage.blobStoreName'],
            fieldConfig: {
              'storage.blobStoreName': { label: RepositoriesPageStrings.FORM_MACHINE.BLOB_STORE, type: 'select' },
              'storage.strictContentTypeValidation': {
                label: RepositoriesPageStrings.FORM_MACHINE.STRICT_CONTENT_TYPE_VALIDATION,
                type: 'checkbox',
              },
              'storage.writePolicy': { label: RepositoriesPageStrings.FORM_MACHINE.DEPLOYMENT_POLICY, type: 'select' },
              'component.proprietaryComponents': {
                label: RepositoriesPageStrings.FORM_MACHINE.PROPRIETARY_COMPONENTS,
                type: 'checkbox',
              },
              'cleanup.policyNames': { label: RepositoriesPageStrings.FORM_MACHINE.CLEANUP_POLICIES, type: 'multiselect' },
            },
          },
        },
        [REPOSITORY_TYPES.PROXY]: {
          meta: {
            typeLabel: RepositoriesPageStrings.FORM_MACHINE.PROXY,
            fields: [
              'storage.blobStoreName',
              'storage.strictContentTypeValidation',
              'proxy.remoteUrl',
              'proxy.contentMaxAge',
              'proxy.metadataMaxAge',
              'negativeCache.enabled',
              'negativeCache.timeToLive',
              'httpClient.blocked',
              'httpClient.autoBlock',
              'httpClient.authentication',
              'routingRuleId',
              'cleanup.policyNames',
              'replication.preemptivePullEnabled',
            ],
            requiredFields: ['storage.blobStoreName', 'proxy.remoteUrl'],
            fieldConfig: {
              'storage.blobStoreName': { label: RepositoriesPageStrings.FORM_MACHINE.BLOB_STORE, type: 'select' },
              'storage.strictContentTypeValidation': {
                label: RepositoriesPageStrings.FORM_MACHINE.STRICT_CONTENT_TYPE_VALIDATION,
                type: 'checkbox',
              },
              'proxy.remoteUrl': {
                label: RepositoriesPageStrings.FORM_MACHINE.REMOTE_STORAGE_URL,
                type: 'text',
                helpText: RepositoriesPageStrings.FORM_MACHINE.LOCATION_REMOTE_REPOSITORY_BEING_PROXIED,
              },
              'proxy.contentMaxAge': { label: RepositoriesPageStrings.FORM_MACHINE.MAXIMUM_COMPONENT_AGE, type: 'number' },
              'proxy.metadataMaxAge': { label: RepositoriesPageStrings.FORM_MACHINE.MAXIMUM_METADATA_AGE, type: 'number' },
              'negativeCache.enabled': { label: RepositoriesPageStrings.FORM_MACHINE.NOT_FOUND_CACHE_ENABLED, type: 'checkbox' },
              'negativeCache.timeToLive': { label: RepositoriesPageStrings.FORM_MACHINE.NOT_FOUND_CACHE_TTL, type: 'number' },
              'httpClient.blocked': { label: RepositoriesPageStrings.FORM_MACHINE.BLOCK_OUTBOUND_CONNECTIONS, type: 'checkbox' },
              'httpClient.autoBlock': {
                label: RepositoriesPageStrings.FORM_MACHINE.AUTO_BLOCK_OUTBOUND_CONNECTIONS,
                type: 'checkbox',
              },
              'httpClient.authentication': { label: RepositoriesPageStrings.FORM_MACHINE.AUTHENTICATION, type: 'fieldset' },
              'routingRuleId': { label: RepositoriesPageStrings.FORM_MACHINE.ROUTING_RULE, type: 'select' },
              'cleanup.policyNames': { label: RepositoriesPageStrings.FORM_MACHINE.CLEANUP_POLICIES, type: 'multiselect' },
              'replication.preemptivePullEnabled': { label: RepositoriesPageStrings.FORM_MACHINE.PREEMPTIVE_PULL, type: 'checkbox' },
            },
          },
        },
        [REPOSITORY_TYPES.GROUP]: {
          meta: {
            typeLabel: RepositoriesPageStrings.FORM_MACHINE.GROUP,
            fields: [
              'storage.blobStoreName',
              'storage.strictContentTypeValidation',
              'group.memberNames',
              'group.writableMember',
              'routingRuleId',
            ],
            requiredFields: ['storage.blobStoreName', 'group.memberNames'],
            fieldConfig: {
              'storage.blobStoreName': { label: RepositoriesPageStrings.FORM_MACHINE.BLOB_STORE, type: 'select' },
              'storage.strictContentTypeValidation': {
                label: RepositoriesPageStrings.FORM_MACHINE.STRICT_CONTENT_TYPE_VALIDATION,
                type: 'checkbox',
              },
              'group.memberNames': {
                label: RepositoriesPageStrings.FORM_MACHINE.MEMBER_REPOSITORIES,
                type: 'transfer',
                helpText: RepositoriesPageStrings.FORM_MACHINE.SELECT_REPOSITORIES_INCLUDE_GROUP,
              },
              'group.writableMember': {
                label: RepositoriesPageStrings.FORM_MACHINE.WRITABLE_MEMBER,
                type: 'select',
                helpText: RepositoriesPageStrings.FORM_MACHINE.SELECT_HOSTED_MEMBER_DEPLOY,
              },
              'routingRuleId': { label: RepositoriesPageStrings.FORM_MACHINE.ROUTING_RULE, type: 'select' },
            },
          },
        },
      },
    },
  });
}
