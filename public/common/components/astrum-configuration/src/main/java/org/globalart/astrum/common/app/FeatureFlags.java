
package org.globalart.astrum.common.app;

/**
 * List of available feature flags
 * You can change it's values by editing ${data-dir}/astrum.properties configuration file.
 *
 * @since 3.20
 */
public class FeatureFlags
{
  /* Cargo format is temporarily hidden behind the feature flag. Default value: false */
  public static final String CARGO_FORMAT_ENABLED = "astrum.format.cargo.enabled";

  /* Hugging Face format is temporarily hidden behind the feature flag. Default value: false */
  public static final String HUGGING_FACE_FORMAT_ENABLED = "astrum.format.huggingface.enabled";

  /*
   * Kill switch for the Hugging Face XET protocol (§0.6, §5 of xet-implementation-spec).
   * Independent of HUGGING_FACE_FORMAT_ENABLED so operators can run the HF format on the
   * legacy LFS-only download path while the XET protocol is disabled. Default value: false.
   * When false the recipe skips registering XET routes and the proxy facet does not persist
   * or emit the X-Xet-Hash content-attribute; instance behaviour is bit-for-bit identical to
   * a pre-XET Astrum.
   */
  public static final String HUGGING_FACE_XET_ENABLED = "astrum.format.huggingface.xet.enabled";

  public static final String HUGGING_FACE_XET_ENABLED_NAMED_VALUE =
      "${" + HUGGING_FACE_XET_ENABLED + ":-false}";

  /* Composer format is temporarily hidden behind the feature flag. Default value: false */
  public static final String COMPOSER_FORMAT_ENABLED = "astrum.format.composer.enabled";

  /*
   * Kill switch for the OCI (Open Container Initiative) repository format. When false, the OCI
   * hosted/proxy/group recipes are not registered, so OCI repositories cannot be created or served.
   * Intended as a field-rollback escape hatch. Available values: true, false. Default value: true.
   */
  public static final String OCI_FORMAT_ENABLED = "astrum.format.oci.enabled";

  public static final String OCI_FORMAT_ENABLED_NAMED_VALUE = "${astrum.format.oci.enabled:true}";

  /*
   * Feature flag for Chocolatey-specific NuGet behaviour (LegacyGallery suppression
   * in v3 hosted service index, and mixed v2+v3 group routing). When true, enables
   * Chocolatey support. Available values: true, false. Default value: true.
   */
  public static final String NUGET_CHOCOLATEY_ENABLED = "astrum.nuget.chocolatey.enabled";

  public static final String NUGET_CHOCOLATEY_ENABLED_NAMED_VALUE = "${astrum.nuget.chocolatey.enabled:true}";

  /* Docker GC Custom task enabled. Available values: true, false. Default value: false */
  public static final String DOCKER_GC_CUSTOM_TASK_ENABLED = "astrum.docker.gc.custom.enabled";

  /* Database externalization developers only. Available values: true, false. Default value: false */
  public static final String DATASTORE_DEVELOPER = "astrum.datastore.developer";

  public static final String DATASTORE_DEVELOPER_NAMED_VALUE = "${astrum.datastore.developer:false}";

  /* Distributed event service. Available values: true, false. Default value: false */
  public static final String DATASTORE_CLUSTERED_ENABLED = "astrum.datastore.clustered.enabled";

  public static final String DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE = "${astrum.datastore.clustered.enabled:false}";

  /* Zero downtime upgrades while clustered. Available values: true, false. Default value: false */
  public static final String CLUSTERED_ZERO_DOWNTIME_ENABLED = "astrum.zero.downtime.enabled";

  public static final String CLUSTERED_ZERO_DOWNTIME_ENABLED_NAMED_VALUE = "${astrum.zero.downtime.enabled:false}";

  public static final String CLUSTERED_ZERO_DOWNTIME_ENABLED_ENV = "ASTRUM_ZERO_DOWNTIME_ENABLED";

  /**
   * Heartbeat interval in seconds for cluster node health monitoring.
   * <p>
   * Default: 600 seconds (10 minutes). This interval determines:
   * <ul>
   * <li>How frequently nodes write heartbeat data to the database</li>
   * <li>The maximum time before a failed node's stale heartbeat is considered inactive</li>
   * <li>The timeout for Zero Downtime Upgrade coordination (all nodes must reach consensus within this window)</li>
   * </ul>
   */
  public static final String HEARTBEAT_INTERVAL_SECONDS = "astrum.heartbeat.interval";

  public static final int HEARTBEAT_INTERVAL_SECONDS_DEFAULT = 600;

  /* Feature flag to indicate if current db is postgresql */
  public static final String DATASTORE_IS_POSTGRESQL = "datastore.isPostgresql";

  /* JWT externalization. Available values: true, false. Default value: true unless astrum.session.enabled=true */
  public static final String JWT_ENABLED = "astrum.jwt.enabled";

  /* Session flag for marking content that is only for session, and should be disabled when jwt is enabled */
  public static final String SESSION_ENABLED = "astrum.session.enabled";

  /* HTTP Replication. Available values: true, false. Default value: true */
  public static final String REPLICATION_HTTP_ENABLED = "astrum.replication.http.enabled";

  /*
   * flag for skipping blob store with soft-quota violation (for Round Robin group policy)
   * Available values: true, false. Default value: false
   */
  public static final String BLOBSTORE_SKIP_ON_SOFTQUOTA_VIOLATION = "astrum.blobstore.skipOnSoftQuotaViolation";

  /*  */
  public static final String DATASTORE_BLOBSTORE_METRICS = "astrum.datastore.blobstore.metrics.enabled";

  /**
   * The Key-Value DB storage which can be used as a distributed cache. Use it intelligently,
   * for example it makes sense to cache IQ results in a DB rather than request IQ Server each time.
   * At best should be replaced by Redis cache or Apache Ignite or other.
   */
  public static final String SQL_DISTRIBUTED_CACHE = "astrum.datastore.sql.cache.enabled";

  /**
   * Validates attribute from the node_heartbeat.node_info to determine if the deployment is valid.
   */
  public static final String DATASTORE_DEPLOYMENT_VALIDATOR = "astrum.datastore.deployment.validator.enabled";

  public static final String CHANGE_REPO_BLOBSTORE_TASK_ENABLED = "astrum.change.repo.blobstore.task.enabled";

  public static final String CHANGE_REPO_BLOBSTORE_TASK_ENABLED_NAMED_VALUE =
      "${astrum.change.repo.blobstore.task.enabled:false}";

  /**
   * Feature flag to enable/disable RecalculateBlobStoreSizeTask
   */
  public static final String RECALCULATE_BLOBSTORE_SIZE_TASK_ENABLED = "astrum.recalculate.blobstore.size.task.enabled";

  public static final String RECALCULATE_BLOBSTORE_SIZE_TASK_ENABLED_NAMED_VALUE =
      "${" + RECALCULATE_BLOBSTORE_SIZE_TASK_ENABLED + ":true}";

  /**
   * Feature flag to enable/disable the audit-events cleanup task (retention pruning).
   */
  public static final String AUDIT_EVENTS_CLEANUP_TASK_ENABLED = "astrum.audit.events.cleanup.task.enabled";

  public static final String AUDIT_EVENTS_CLEANUP_TASK_ENABLED_NAMED_VALUE =
      "${" + AUDIT_EVENTS_CLEANUP_TASK_ENABLED + ":true}";

  public static final String FIREWALL_ONBOARDING_ENABLED = "astrum.firewall.onboarding.enabled";

  public static final String CLEANUP_PREVIEW_ENABLED = "astrum.cleanup.preview.enabled";

  public static final String CLEANUP_PREVIEW_ENABLED_NAMED_VALUE = "${astrum.cleanup.preview.enabled:true}";

  public static final String CLEANUP_MAVEN_RETAIN = "astrum.cleanup.mavenRetain";

  public static final String CLEANUP_DOCKER_RETAIN = "astrum.cleanup.dockerRetain";

  public static final String CLEANUP_USE_SQL = "astrum.cleanup.useSQL";

  public static final String CLEANUP_RETAIN_ALL_FORMATS = "astrum.cleanup.retainAllFormats.enabled";

  public static final String CLEANUP_RETAIN_ALL_FORMATS_NAMED_VALUE = "${astrum.cleanup.retainAllFormats.enabled:true}";

  public static final String FORMAT_RETAIN_PATTERN = "astrum.cleanup.{format}Retain";

  public static final String DISABLE_NORMALIZE_VERSION_TASK = "astrum.cleanup.disableNormalizeVersionTask";

  public static final String DISABLE_RENORMALIZE_VERSION_TASK = "astrum.cleanup.disableReNormalizeVersionTask";

  public static final String DISABLE_CREATING_COMPONENT_INDEXES_TASK = "astrum.component.index.task";

  public static final String FIREWALL_QUARANTINE_FIX_ENABLED = "astrum.firewall.quarantineFix.enabled";

  public static final String FIREWALL_QUARANTINE_FIX_ENABLED_NAMED_VALUE =
      "${astrum.firewall.quarantineFix.enabled:false}";

  public static final String REACT_PRIVILEGES = "astrum.react.privileges";

  public static final String REACT_PRIVILEGES_NAMED_VALUE = "${astrum.react.privileges:true}";

  public static final String REACT_PRIVILEGES_MODAL_ENABLED = "astrum.react.privileges.modal.enabled";

  public static final String REACT_PRIVILEGES_MODAL_NAMED_VALUE = "${astrum.react.privileges.modal.enabled:true}";

  /**
   * Feature flag to determine if we should include the repository sizes feature
   */
  public static final String REPOSITORY_SIZE_ENABLED = "astrum.repository.size";

  public static final String REPOSITORY_SIZE_ENABLED_NAMED_VALUE = "${astrum.repository.size:true}";

  public static final String CONTENT_USAGE_ENABLED_NAMED_VALUE = "${astrum.contentUsageMetrics.enabled:true}";

  public static final String REACT_ROLES_MODAL_ENABLED = "astrum.react.roles.modal.enabled";

  public static final String REACT_ROLES_MODAL_NAMED_VALUE = "${astrum.react.roles.modal.enabled:true}";

  public static final String BLOBSTORE_OWNERSHIP_CHECK_DISABLED_NAMED_VALUE =
      "${astrum.blobstore.s3.ownership.check.disabled:false}";

  public static final String STARTUP_TASKS_DELAY_SECONDS_VALUE = "${astrum.startup.task.delay.seconds:0}";

  /**
   * Feature flag to expose H2 export database to script task
   */
  public static final String H2_DATABASE_EXPORT_SCRIPT_TASK_ENABLED = "astrum.database.export.script.task.h2.enabled";

  /* When false skips the orient not supported error. Available values: true, false. Default value: true */
  public static final String ORIENT_WARNING = "astrum.orient.warning";

  public static final String ORIENT_WARNING_NAMED_VALUE = "${astrum.orient.warning:true}";

  /**
   * When true (default), the Secure attribute will be set on the ASTRUMSESSIONID Cookie when delivered over https.
   * In deployments with HTTP-only listeners, this setting will typically have no effect.
   * Setting false for this property in HTTPS only environments is not recommended.
   *
   * See https://owasp.org/www-community/controls/SecureCookieAttribute
   */
  public static final String ASTRUMSESSIONID_SECURE_COOKIE_NAMED_VALUE = "${astrum.session.secureCookie:true}";

  public static final String ASSET_AUDITOR_ATTRIBUTE_CHANGES_ENABLED_VALUE =
      "${astrum.audit.attribute.changes.enabled:true}";

  public static final String ZERO_DOWNTIME_MARKETING_MODAL_ENABLED = "zero.downtime.marketing.modal";

  public static final String ZERO_DOWNTIME_MARKETING_MODAL_ENABLED_NAMED_VALUE =
      "${zero.downtime.marketing.modal:false}";

  /* For testing purposes only */
  public static final String ZERO_DOWNTIME_BASELINE_FAIL = "astrum.zdu.baseline.fail";

  /* For testing purposes only */
  public static final String ZERO_DOWNTIME_FUTURE_MIGRATION_ENABLED = "astrum.zdu.future.enabled";

  public static final String MALWARE_RISK_ENABLED = "astrum.malware.risk.enabled";

  public static final String MALWARE_RISK_ENABLED_NAMED_VALUE = "${astrum.malware.risk.enabled:true}";

  public static final String MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED =
      "astrum.malware.risk.on.disk.nonadmin.override.enabled";

  public static final String MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED_NAMED_VALUE =
      "${astrum.malware.risk.on.disk.nonadmin.override.enabled:false}";

  public static final String MALWARE_REMEDIATOR_TASK_CHECK_REPOSITORY_IN_KNOWN_REGISTRIES_NAMED_VALUE =
      "${astrum.malware.remediator.task.check.repository.in.known.registries:false}";

  public static final String MALWARE_REMEDIATOR_TASK_IGNORE_QUARANTINE_STATE_NAMED_VALUE =
      "${astrum.malware.remediator.task.ignore.quarantine.state:true}";

  /* properties/env vars used by secrets service */
  public static final String SECRETS_FILE = "astrum.secrets.file";

  public static final String SECRETS_FILE_ENV = "ASTRUM_SECRETS_KEY_FILE";

  public static final String RECONCILE_CLEANUP_DAYS_AGO_VALUE = "${astrum.reconcile.cleanup.daysAgo:7}";

  public static final String SECRETS_API_ENABLED = "astrum.secrets.api.enabled";

  public static final String ASTRUM_SECURITY_OAUTH2_ENABLED = "astrum.security.oauth2.enabled";

  public static final String ASTRUM_SECURITY_FIPS_ENABLED = "astrum.security.fips.enabled";

  public static final String ASTRUM_SECURITY_FIPS_ENABLED_NAMED_VALUE = "${astrum.security.fips.enabled:false}";

  public static final String ASTRUM_SECURITY_PASSWORD_ALGORITHM_NAMED_VALUE =
      "${astrum.security.password.algorithm:shiro1}";

  public static final String ASTRUM_SECURITY_PASSWORD_ITERATIONS_NAMED_VALUE =
      "${astrum.security.password.iterations:}";

  /*
   * Short-lived cache of successfully verified credentials, keyed by a keyed hash of (stored-hash + submitted
   * password). Lets repeated identical Basic-Auth requests skip the deliberately-expensive password KDF. Only
   * successful verifications are cached (failures always pay the full KDF cost, preserving brute-force resistance),
   * and the stored-hash is part of the key so a password change transparently invalidates prior entries.
   */
  public static final String ASTRUM_SECURITY_PASSWORD_CACHE_ENABLED_NAMED_VALUE =
      "${astrum.security.password.cache.enabled:true}";

  public static final String ASTRUM_SECURITY_PASSWORD_CACHE_SIZE_NAMED_VALUE =
      "${astrum.security.password.cache.size:1000}";

  public static final String ASTRUM_SECURITY_PASSWORD_CACHE_EXPIRE_SECONDS_NAMED_VALUE =
      "${astrum.security.password.cache.expireSeconds:2}";

  public static final String ASTRUM_SECURITY_SECRETS_ALGORITHM_NAMED_VALUE =
      "${astrum.security.secrets.algorithm:PBKDF2WithHmacSHA1}";

  public static final String ASTRUM_SECURITY_SECRETS_ITERATIONS_NAMED_VALUE =
      "${astrum.security.secrets.iterations:}";

  /* Service Account tokens feature. Available values: true, false. Default value: false */
  public static final String SERVICE_ACCOUNT_ENABLED = "astrum.service.account.enabled";

  public static final String SERVICE_ACCOUNT_ENABLED_NAMED_VALUE = "${" + SERVICE_ACCOUNT_ENABLED + ":false}";

  public static final String CONTAINER_IMAGES_EVAL_ENABLED = "astrum.container.images.eval.enabled";

  public static final String CONTAINER_IMAGES_EVAL_ENABLED_NAMED = "${astrum.container.images.eval.enabled:-true}";

  public static final String CONTAINER_IMAGES_EVAL_ENABLED_NAMED_VALUE = "${astrum.container.images.eval.enabled:true}";

  public static final String ASTRUM_SECURITY_AUTH0_USER_MANAGEMENT_ENABLED =
      "astrum.security.auth0.userManagement.enabled";

  public static final String FIREWALL_CONTAINER_WORK_DIRECTORY_VALUE = "${astrum.firewall.container.workdirectory:}";

  /* Layer download timeout for container image scanning, in minutes. When unset, the scanner default is used. */
  public static final String FIREWALL_CONTAINER_DOWNLOAD_TIMEOUT_MINUTES =
      "astrum.firewall.container.download.timeout.minutes";

  public static final String FIREWALL_CONTAINER_DOWNLOAD_TIMEOUT_MINUTES_VALUE =
      "${astrum.firewall.container.download.timeout.minutes:}";

  public static final String EGRESS_METRICS_AGGREGATION_TASK_VISIBLE = "${astrum.egressmetrics.task.visible:false}";

  /*
   * PyPi metadata features (PEP 658 + 691). Enables metadata distribution and JSON API. Available values: true, false.
   * Default value: false
   */
  public static final String PYPI_METADATA_ENABLED = "astrum.pypi.metadata.enabled";

  public static final String PYPI_METADATA_ENABLED_NAMED_VALUE = "${astrum.pypi.metadata.enabled:false}";

  /*
   * PyPI repair-metadata-content-type task visibility. When true, the task appears in the Tasks UI so
   * operators can monitor and re-run it. Default: false (hidden; auto-scheduled via migration step).
   */
  public static final String PYPI_REPAIR_METADATA_CONTENT_TYPE_TASK_VISIBLE =
      "astrum.pypi.repair.metadata.content.type.task.visible";

  public static final String PYPI_REPAIR_METADATA_CONTENT_TYPE_TASK_VISIBLE_NAMED_VALUE =
      "${" + PYPI_REPAIR_METADATA_CONTENT_TYPE_TASK_VISIBLE + ":false}";

  public static final String ASTRUM_USER_CONFIGURATION_SOURCE_ENABLED = "astrum.user.configuration.source.enabled";

  /* ExtJS Capabilities Page. Available values: true, false. Default value: false */
  public static final String EXTJS_CAPABILITIES_ENABLED = "astrum.extjs.capabilities.enabled";

  public static final String EXTJS_CAPABILITIES_NAMED_VALUE = "${astrum.extjs.capabilities.enabled:false}";

  /* React Capabilities Page. Available values: true, false. Default value: true */
  public static final String REACT_CAPABILITIES_ENABLED = "astrum.react.capabilities.enabled";

  public static final String REACT_CAPABILITIES_NAMED_VALUE = "${astrum.react.capabilities.enabled:true}";

  /*
   * Gates the new React onboarding wizard. When enabled, the ExtJS onboarding wizard defers so the React wizard can
   * take over. Available values: true, false. Default value: true
   */
  public static final String REACT_ONBOARDING_ENABLED = "astrum.react.onboarding.enabled";

  public static final String REACT_ONBOARDING_ENABLED_NAMED_VALUE = "${astrum.react.onboarding.enabled:true}";

  /* Enable principal permissions cache. Default value: true */
  public static final String PRINCIPAL_PERMISSIONS_CACHE_ENABLED_NAMED_VALUE =
      "${astrum.security.principal.permissions.cache.enabled:true}";

  /* Preview UI for anonymous users. The UI state always reports true; the value only seeds the
   * Preview UI settings REST resource. Available values: true, false. Default value: true */
  public static final String PREVIEW_UI_ANONYMOUS_ENABLED = "astrum.preview.ui.anonymous.enabled";

  public static final String PREVIEW_UI_ANONYMOUS_ENABLED_NAMED_VALUE =
      "${astrum.preview.ui.anonymous.enabled:true}";

  /* Preview UI for logged-in users. The UI state always reports true; the value only seeds the
   * Preview UI settings REST resource. Available values: true, false. Default value: true */
  public static final String PREVIEW_UI_LOGGEDIN_ENABLED = "astrum.preview.ui.loggedin.enabled";

  public static final String PREVIEW_UI_LOGGEDIN_ENABLED_NAMED_VALUE =
      "${astrum.preview.ui.loggedin.enabled:true}";

  /* Default to Preview UI for all users. The UI state always reports true; the value only seeds the
   * Preview UI settings REST resource. Available values: true, false. Default value: true */
  public static final String PREVIEW_UI_DEFAULT_ENABLED = "astrum.preview.ui.default.enabled";

  public static final String PREVIEW_UI_DEFAULT_ENABLED_NAMED_VALUE =
      "${astrum.preview.ui.default.enabled:true}";

  /* Disable Legacy UI (ExtJS) for all users. The UI state always reports true; the value only seeds
   * the Preview UI settings REST resource. Available values: true, false. Default value: true */
  public static final String PREVIEW_UI_LEGACY_DISABLED = "astrum.preview.ui.legacy.disabled";

  public static final String PREVIEW_UI_LEGACY_DISABLED_NAMED_VALUE =
      "${astrum.preview.ui.legacy.disabled:true}";

  /* Disable Classic UI switch feedback collection. Available values: true, false. Default value: false */
  public static final String PREVIEW_UI_SWITCH_FEEDBACK_DISABLED =
      "astrum.preview.ui.switch.feedback.disabled";

  public static final String PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_NAMED_VALUE =
      "${astrum.preview.ui.switch.feedback.disabled:false}";

  public static final String PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_KEY =
      "preview.ui.switch.feedback.disabled";

  /*
   * Registers the Preview UI settings and Classic switch feedback REST endpoints.
   * The endpoints Astrum One reads (browse, roles, API permissions, API access check, suggest) do
   * not depend on it.
   * Available values: true, false. Default value: true
   */
  public static final String PREVIEW_UI_SETTINGS_ENABLED = "astrum.previewui.enabled";

  public static final String PREVIEW_UI_SETTINGS_ENABLED_NAMED_VALUE = "${astrum.previewui.enabled:true}";

  /*
   * Gates Preview UI audit event DB persistence and the audit log REST/UI surface.
   * Intentionally decoupled from the preview UI visibility flags so audit DB writes can be
   * deferred until the persistence path is production-ready.
   * Available values: true, false. Default value: true
   */
  public static final String PREVIEW_UI_AUDIT_ENABLED = "astrum.previewui.audit.enabled";

  public static final String PREVIEW_UI_AUDIT_ENABLED_NAMED_VALUE = "${astrum.previewui.audit.enabled:true}";

  /*
   * Per-realm principal permissions cache maximum size. Default value: 1000. Override per realm by appending the
   * realm name, e.g. astrum.authorizingrealm.permissionscache.maximumsize.LdapRealm=5000.
   */
  public static final String PRINCIPAL_PERMISSIONS_CACHE_MAXIMUM_SIZE =
      "astrum.authorizingrealm.permissionscache.maximumsize";

  public static final String PRINCIPAL_PERMISSIONS_CACHE_MAXIMUM_SIZE_NAMED_VALUE =
      "${astrum.authorizingrealm.permissionscache.maximumsize:1000}";

  /* Principal permissions cache record statistics. Default value: false (enable when diagnosing). */
  public static final String PRINCIPAL_PERMISSIONS_CACHE_RECORD_STATS =
      "astrum.authorizingrealm.permissionscache.recordstats";

  public static final String PRINCIPAL_PERMISSIONS_CACHE_RECORD_STATS_NAMED_VALUE =
      "${astrum.authorizingrealm.permissionscache.recordstats:false}";

  /* Principal permissions cache concurrency level. Default value: 16 */
  public static final String PRINCIPAL_PERMISSIONS_CACHE_CONCURRENCY_LEVEL =
      "astrum.authorizingrealm.permissionscache.concurrencylevel";

  public static final String PRINCIPAL_PERMISSIONS_CACHE_CONCURRENCY_LEVEL_NAMED_VALUE =
      "${astrum.authorizingrealm.permissionscache.concurrencylevel:16}";

  // Hosted Repository Evaluation feature. Default value: false
  public static final String HOSTED_REPOSITORY_EVALUATION_ENABLED = "astrum.hosted.repository.evaluation.enabled";

  public static final String HOSTED_REPOSITORY_EVALUATION_ENABLED_NAMED_VALUE =
      "${astrum.hosted.repository.evaluation.enabled:false}";

  public static final String HOSTED_REPOSITORY_WORK_DIRECTORY_VALUE =
      "${astrum.lifecycle.hosted-repository.workdirectory:}";

  /**
   * Controls whether query parameter forwarding is available for Raw proxy repositories.
   * When enabled, the UI configuration panel and REST API fields ({@code forwardQueryParameters},
   * {@code excludedQueryParameters}) become visible and functional. When disabled (the default),
   * the feature is completely hidden: the UI panel is suppressed and the REST API ignores
   * these attributes on both input and output.
   *
   * Default: false (feature hidden)
   */
  public static final String RAW_QUERYPARAMS_FORWARDING_ENABLED = "astrum.raw.queryparams.forwarding.enabled";

  public static final String RAW_QUERYPARAMS_FORWARDING_ENABLED_NAMED_VALUE =
      "${astrum.raw.queryparams.forwarding.enabled:false}";

  /**
   * Controls whether internal JVM/infrastructure metrics (JVM gauges, Jetty metrics, HikariCP metrics, thread dumps,
   * HTTP request metrics, and @Timed/@ExceptionMetered aspects) are registered. When disabled, the metrics endpoints
   * remain available but only expose custom application gauges. Default: true (all metrics enabled).
   */
  public static final String METRICS_INTERNAL_ENABLED = "astrum.metrics.internal.enabled";

  public static final String METRICS_INTERNAL_ENABLED_NAMED_VALUE = "${astrum.metrics.internal.enabled:true}";

  /**
   * Default is false until ASTRUM-49817 development is complete.
   */
  public static final String TELEMETRY_MANDATORY_ENABLED = "astrum.telemetry.mandatory.enabled";

  /**
   * When enabled, activates mandatory telemetry logic for alerting only, without blocking operations (e.g., read-only
   * mode). Default value: true
   */
  public static final String TELEMETRY_MANDATORY_WARNING_ENABLED = "astrum.telemetry.mandatory.warning.enabled";

  /*
   * Authentication rate limiting (brute force protection, CWE-307). Available values: true, false. Default value: true
   */
  public static final String AUTH_RATE_LIMIT_ENABLED = "astrum.auth.ratelimit.enabled";

  public static final String AUTH_RATE_LIMIT_ENABLED_NAMED_VALUE = "${astrum.auth.ratelimit.enabled:true}";

  /* NuGet Symbol Server support. Available values: true, false. Default value: true */
  public static final String NUGET_SYMBOL_SERVER_ENABLED = "astrum.nuget.symbol.server.enabled";

  public static final String NUGET_SYMBOL_SERVER_ENABLED_NAMED_VALUE = "${astrum.nuget.symbol.server.enabled:true}";

  /* Firewall capability shim for backwards API compatibility. Available values: true, false. Default value: true */
  public static final String FIREWALL_CAPABILITY_SHIM_ENABLED = "astrum.firewall.capability.shim.enabled";

  /*
   * Kill switch for the new repository-config-driven firewall lifecycle (FirewallRepositoryService
   * — audit task scheduling, online/offline + IQ-capability transition handling, RepositoryUpdated/
   * Created/Deleted event handling). Setting this to false disables the new lifecycle entirely;
   * existing repository firewall.mode configuration remains in storage but is dormant. Intended
   * as a field-rollback escape hatch for the migration window. Available values: true, false.
   * Default value: true.
   */
  public static final String FIREWALL_REPOSITORY_SERVICE_ENABLED = "astrum.firewall.repository.service.enabled";

  /*
   * FIRE-105 / ASTRUM-52802 — PyPI PCCS throughput perf fixes (template content cache, Guava HTML escaper,
   * pre-escape relocation + safeLink scheme guard, reflection cache). Gates the optimised PyPI
   * simple-index rendering path; default true (optimised path on unless explicitly
   * disabled). Set false to run the legacy path. Available values: true, false.
   */
  public static final String ASTRUM_PCCS_PERF_PYPI_ENABLED = "astrum.pccs.perf.pypi.enabled";

  public static final String ASTRUM_PCCS_PERF_PYPI_ENABLED_NAMED_VALUE = "${astrum.pccs.perf.pypi.enabled:true}";

  /*
   * ASTRUM-52802 — npm PCCS throughput perf fixes (shared ObjectMapper singleton + ThreadLocal,
   * reflection cache). Gates the optimised npm metadata streaming path; default true (optimised
   * path on unless explicitly disabled). Set false to run the legacy per-request path. Available
   * values: true, false.
   */
  public static final String ASTRUM_PCCS_PERF_NPM_ENABLED = "astrum.pccs.perf.npm.enabled";

  public static final String ASTRUM_PCCS_PERF_NPM_ENABLED_NAMED_VALUE = "${astrum.pccs.perf.npm.enabled:true}";
}
