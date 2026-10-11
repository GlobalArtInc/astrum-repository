import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';


/**
 * Descriptions for all task types.
 * Used by TaskTypeSelector to display helpful context about each task type.
 */
export const TASK_TYPE_DESCRIPTIONS: Record<string, string> = {
  // Admin tasks
  'db.backup': TasksStrings.TYPE_DESCRIPTIONS.CREATES_BACKUP_EMBEDDED_DATABASE_SPECIFIED,
  'blobstore.compact': TasksStrings.TYPE_DESCRIPTIONS.RECLAIMS_DISK_SPACE_PERMANENTLY_REMOVING,
  'blobstore.delete-temp-files': TasksStrings.TYPE_DESCRIPTIONS.DELETES_TEMPORARY_FILES_LEFT_BEHIND,
  'blobstore.rebuildComponentDB': TasksStrings.TYPE_DESCRIPTIONS.REBUILDS_COMPONENT_DATABASE_BLOB_STORE,
  'blobstore.move': TasksStrings.TYPE_DESCRIPTIONS.MOVES_ALL_REPOSITORY_DATA_ONE,
  'blobstore.removeMemberFromGroup': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_MEMBER_BLOB_STORE_BLOB,
  'blobstore.planReconciliation': TasksStrings.TYPE_DESCRIPTIONS.GENERATES_RECOVERY_PLAN_RECONCILE_BLOB,
  'blobstore.executeReconciliationPlan': TasksStrings.TYPE_DESCRIPTIONS.EXECUTES_RECOVERY_PLAN_GENERATED_DATA,
  'security.purge-api-keys': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_EXPIRED_ORPHANED_API_KEYS,
  'usertoken.flush': TasksStrings.TYPE_DESCRIPTIONS.INVALIDATES_ALL_ACTIVE_USER_TOKENS,
  'h2.backup': TasksStrings.TYPE_DESCRIPTIONS.CREATES_BACKUP_H2_EMBEDDED_DATABASE,
  // Descriptor TYPE_ID is `h2.backup.task` (H2BackupTaskDescriptor) — only exposed on H2
  // deployments. Without this key getTaskTypeDescription falls back to the generic message.
  'h2.backup.task': TasksStrings.TYPE_DESCRIPTIONS.CREATES_BACKUP_EMBEDDED_H2_DATABASE,

  // Repair tasks
  'blobstore.metrics.reconcile': TasksStrings.TYPE_DESCRIPTIONS.RECALCULATES_STORAGE_SIZE_METRICS_SELECTED,

  // Cloud tasks
  'astrum.cloud.blobstore.removal': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_UNUSED_BLOB_STORES_CLOUD,

  // Repository tasks
  'repository.cleanup': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_COMPONENTS_MATCHING_CLEANUP_POLICY,
  'repository.rebuild-index': TasksStrings.TYPE_DESCRIPTIONS.REBUILDS_SEARCH_INDEX_SELECTED_REPOSITORY,
  'repository.delete-incomplete-uploads': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_PARTIALLY_UPLOADED_FILES_WERE,
  'repository.purge-unused': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_PROXY_REPOSITORY_COMPONENTS_HAVE,

  // Cleanup tasks
  'assetBlob.cleanup': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_ORPHANED_ASSET_BLOBS_NO,

  // Maven tasks
  'repository.maven.rebuild-metadata': TasksStrings.TYPE_DESCRIPTIONS.REGENERATES_MAVEN_METADATA_FILES_MAVEN,
  'repository.maven.remove-snapshots': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_OLD_SNAPSHOT_VERSIONS_BASED,
  'repository.maven.unpublish-dotIndex': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_LEGACY_INDEX_FILES_MAVEN,
  'repository.maven.publish-dotindex': TasksStrings.TYPE_DESCRIPTIONS.PUBLISHES_MAVEN_INDEX_FILES_REPOSITORY,
  'repository.maven.purge-unused-snapshots': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_SNAPSHOT_VERSIONS_HAVE_NOT,

  // Docker tasks
  'repository.docker.gc': TasksStrings.TYPE_DESCRIPTIONS.RUNS_GARBAGE_COLLECTION_REMOVE_UNREFERENCED,
  'repository.docker.upload-purge': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_INCOMPLETE_DOCKER_LAYER_UPLOADS,

  // npm tasks
  'repository.npm.rebuild-metadata': TasksStrings.TYPE_DESCRIPTIONS.REGENERATES_NPM_PACKAGE_METADATA_FIX,

  // Health Check tasks
  'healthcheck.vulnerabilityReport': TasksStrings.TYPE_DESCRIPTIONS.GENERATES_VULNERABILITY_REPORT_COMPONENTS_SELECTED,
  'healthcheck.eula': TasksStrings.TYPE_DESCRIPTIONS.ACCEPTS_HEALTH_CHECK_EULA_ENABLE,
  'malicious.components.report': TasksStrings.TYPE_DESCRIPTIONS.SCANS_REPOSITORIES_KNOWN_MALICIOUS_COMPONENTS,
  'malicious.components.cleanup': TasksStrings.TYPE_DESCRIPTIONS.AUTOMATICALLY_REMOVES_COMPONENTS_IDENTIFIED_MALICIOUS,

  // Tags tasks
  'tags.cleanup': TasksStrings.TYPE_DESCRIPTIONS.REMOVES_TAGS_MATCHING_SPECIFIED_PATTERN,

  // Yum tasks
  'repository.yum.rebuild.metadata': TasksStrings.TYPE_DESCRIPTIONS.REGENERATES_YUM_REPOSITORY_METADATA_REPODATA,

  // APT tasks
  'repository.apt.rebuild.metadata': TasksStrings.TYPE_DESCRIPTIONS.REGENERATES_APT_REPOSITORY_METADATA_DEBIAN,

  // Helm tasks
  'repository.helm.rebuild.metadata': TasksStrings.TYPE_DESCRIPTIONS.REGENERATES_HELM_CHART_REPOSITORY_INDEX,

  // R tasks
  'repository.r.rebuild.metadata': TasksStrings.TYPE_DESCRIPTIONS.REGENERATES_R_CRAN_REPOSITORY_PACKAGES,

  // PyPI tasks
  'repository.pypi.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_PYPI_REPOSITORY_METADATA_INDEX,

  // Conda tasks
  'repository.conda.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_CONDA_REPOSITORY_METADATA_REPODATA,

  // RubyGems tasks
  'repository.rubygems.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_RUBYGEMS_REPOSITORY_SPECIFICATIONS_METADATA,
  'repository.ruby.rebuild.versions': TasksStrings.TYPE_DESCRIPTIONS.REBUILDS_VERSIONS_FILE_RUBYGEMS_HOSTED,

  // Alpine tasks
  'repository.alpine.rebuild.metadata': TasksStrings.TYPE_DESCRIPTIONS.REBUILDS_ALPINE_REPOSITORY_METADATA_APKINDEX,

  // Go tasks
  'repository.go.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_GO_MODULE_PROXY_METADATA,

  // P2 tasks
  'repository.p2.rebuild.metadata': TasksStrings.TYPE_DESCRIPTIONS.REGENERATES_ECLIPSE_P2_REPOSITORY_METADATA,

  // Conan tasks
  'repository.conan.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_CONAN_C_C_PACKAGE,

  // NuGet tasks
  'repository.nuget.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_NUGET_NET_PACKAGE_REPOSITORY,

  // Cocoapods tasks
  'repository.cocoapods.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_COCOAPODS_IOS_MACOS_PACKAGE,

  // Raw tasks
  'repository.raw.repair': TasksStrings.TYPE_DESCRIPTIONS.REPAIRS_RAW_REPOSITORY_STRUCTURE_METADATA,

  // Generic/Other tasks
  'script': TasksStrings.TYPE_DESCRIPTIONS.EXECUTES_CUSTOM_GROOVY_SCRIPT_SERVER,
  'statisticsCollection': TasksStrings.TYPE_DESCRIPTIONS.COLLECTS_AGGREGATES_REPOSITORY_USAGE_STATISTICS,
  'storage.facet.cleanup': TasksStrings.TYPE_DESCRIPTIONS.CLEANS_UP_ORPHANED_STORAGE_FACET,
  'storage.facet.reconciliation': TasksStrings.TYPE_DESCRIPTIONS.RECONCILES_STORAGE_FACET_DATA_ACTUAL,
};

/**
 * Category assignments for task types.
 * Groups tasks by functional area in the type selector.
 */
export const TASK_TYPE_CATEGORIES: Record<string, string> = {
  // Admin
  'db.backup': 'Admin',
  'blobstore.compact': 'Admin',
  'blobstore.delete-temp-files': 'Admin',
  'blobstore.rebuildComponentDB': 'Admin',
  'blobstore.move': 'Admin',
  'blobstore.removeMemberFromGroup': 'Admin',
  'blobstore.planReconciliation': 'Admin',
  'blobstore.executeReconciliationPlan': 'Admin',
  'security.purge-api-keys': 'Admin',
  'usertoken.flush': 'Admin',
  'h2.backup': 'Admin',
  'h2.backup.task': 'Admin',

  // Repair
  'blobstore.metrics.reconcile': 'Repair',

  // Cloud
  'astrum.cloud.blobstore.removal': 'Cloud',

  // Repository
  'repository.cleanup': 'Repository',
  'repository.rebuild-index': 'Repository',
  'repository.delete-incomplete-uploads': 'Repository',
  'repository.purge-unused': 'Repository',

  // Cleanup
  'assetBlob.cleanup': 'Cleanup',

  // Maven
  'repository.maven.rebuild-metadata': 'Maven',
  'repository.maven.remove-snapshots': 'Maven',
  'repository.maven.unpublish-dotIndex': 'Maven',
  'repository.maven.publish-dotindex': 'Maven',
  'repository.maven.purge-unused-snapshots': 'Maven',

  // Docker
  'repository.docker.gc': 'Docker',
  'repository.docker.upload-purge': 'Docker',

  // npm
  'repository.npm.rebuild-metadata': 'npm',

  // Health Check
  'healthcheck.vulnerabilityReport': 'Health Check',
  'healthcheck.eula': 'Health Check',
  'malicious.components.report': 'Health Check',
  'malicious.components.cleanup': 'Health Check',

  // Tags
  'tags.cleanup': 'Tags',

  // Format-specific
  'repository.yum.rebuild.metadata': 'Yum',
  'repository.apt.rebuild.metadata': 'APT',
  'repository.helm.rebuild.metadata': 'Helm',
  'repository.r.rebuild.metadata': 'R',
  'repository.pypi.repair': 'PyPI',
  'repository.conda.repair': 'Conda',
  'repository.rubygems.repair': 'RubyGems',
  'repository.ruby.rebuild.versions': 'RubyGems',
  'repository.alpine.rebuild.metadata': 'Alpine',
  'repository.go.repair': 'Go',
  'repository.p2.rebuild.metadata': 'P2',
  'repository.conan.repair': 'Conan',
  'repository.nuget.repair': 'NuGet',
  'repository.cocoapods.repair': 'CocoaPods',
  'repository.raw.repair': 'Raw',

  // Other
  'script': 'Other',
  'statisticsCollection': 'Other',
  'storage.facet.cleanup': 'Other',
  'storage.facet.reconciliation': 'Other',
};

/**
 * Icons for task type categories.
 * Maps category names to Lucide icon names.
 */
export const CATEGORY_ICONS: Record<string, string> = {
  Admin: 'Settings',
  Repository: 'Database',
  Cleanup: 'Trash2',
  Repair: 'Wrench',
  Cloud: 'Cloud',
  Maven: 'Package',
  Docker: 'Container',
  npm: 'Package',
  'Health Check': 'HeartPulse',
  Tags: 'Tag',
  Yum: 'Package',
  APT: 'Package',
  Helm: 'Ship',
  R: 'BarChart',
  PyPI: 'Package',
  Conda: 'Package',
  RubyGems: 'Gem',
  Go: 'Code',
  P2: 'Eclipse',
  Conan: 'Package',
  NuGet: 'Package',
  CocoaPods: 'Apple',
  Raw: 'File',
  Other: 'MoreHorizontal',
};

/**
 * Get description for a task type by ID.
 * Returns a default message if not found.
 */
export function getTaskTypeDescription(typeId: string): string {
  return TASK_TYPE_DESCRIPTIONS[typeId] || TasksStrings.TYPE_DESCRIPTIONS.CONFIGURE_SCHEDULE_TASK_TYPE;
}

const CATEGORY_LABELS: Record<string, string> = {
  Admin: TasksStrings.CATEGORIES.ADMIN,
  Repository: TasksStrings.CATEGORIES.REPOSITORY,
  Cleanup: TasksStrings.CATEGORIES.CLEANUP,
  Repair: TasksStrings.CATEGORIES.REPAIR,
  Cloud: TasksStrings.CATEGORIES.CLOUD,
  'Health Check': TasksStrings.CATEGORIES.HEALTH_CHECK,
  Tags: TasksStrings.CATEGORIES.TAGS,
  Other: TasksStrings.CATEGORIES.OTHER,
};

/**
 * The name of a task category as shown to the user: categories named after a format keep the format's name.
 */
export function getTaskCategoryLabel(category: string): string {
  return CATEGORY_LABELS[category] || category;
}

/**
 * Get category for a task type by ID.
 * Uses pattern matching as fallback if not explicitly mapped.
 */
export function getTaskTypeCategory(typeId: string): string {
  // Check explicit mapping first
  if (TASK_TYPE_CATEGORIES[typeId]) {
    return TASK_TYPE_CATEGORIES[typeId];
  }

  // Pattern-based fallback
  if (typeId.startsWith('repository.maven')) return 'Maven';
  if (typeId.startsWith('repository.docker')) return 'Docker';
  if (typeId.startsWith('repository.npm')) return 'npm';
  if (typeId.startsWith('repository.yum')) return 'Yum';
  if (typeId.startsWith('repository.apt')) return 'APT';
  if (typeId.startsWith('repository.helm')) return 'Helm';
  if (typeId.startsWith('repository.r.')) return 'R';
  if (typeId.startsWith('repository.pypi')) return 'PyPI';
  if (typeId.startsWith('repository.conda')) return 'Conda';
  if (typeId.startsWith('repository.ruby')) return 'RubyGems';
  if (typeId.startsWith('repository.rubygems')) return 'RubyGems';
  if (typeId.startsWith('repository.alpine')) return 'Alpine';
  if (typeId.startsWith('repository.go')) return 'Go';
  if (typeId.startsWith('repository.p2')) return 'P2';
  if (typeId.startsWith('repository.conan')) return 'Conan';
  if (typeId.startsWith('repository.nuget')) return 'NuGet';
  if (typeId.startsWith('repository.cocoapods')) return 'CocoaPods';
  if (typeId.startsWith('repository.raw')) return 'Raw';
  if (typeId.startsWith('repository.')) return 'Repository';
  if (typeId.startsWith('astrum.cloud.')) return 'Cloud';
  if (typeId.startsWith('blobstore.') || typeId.startsWith('db.') || typeId.startsWith('h2.') ||
      typeId.startsWith('security.') || typeId.startsWith('usertoken.')) return 'Admin';
  if (typeId.startsWith('healthcheck') || typeId.startsWith('malicious')) return 'Health Check';
  if (typeId.startsWith('assetBlob.')) return 'Cleanup';
  if (typeId.startsWith('tags.')) return 'Tags';

  return 'Other';
}
