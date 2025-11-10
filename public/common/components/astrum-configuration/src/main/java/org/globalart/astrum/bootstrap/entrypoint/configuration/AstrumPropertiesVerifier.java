
package org.globalart.astrum.bootstrap.entrypoint.configuration;

import static com.google.common.base.Strings.isNullOrEmpty;
import static java.lang.Boolean.parseBoolean;
import static org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration.BASEDIR_SYS_PROP;
import static org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration.DATADIR_SYS_PROP;
import static org.globalart.astrum.common.app.FeatureFlags.*;

/**
 * This class will make sure that the AstrumProperties are set correctly, that there aren't mutually exclusive
 * configurations, and that required properties are present.
 * <p>
 * A reworking of the AstrumEditionPropertiesConfigurer to be used in the new spring-only world
 */
public class AstrumPropertiesVerifier
{
  public static final String FALSE = "false";

  public static final String TRUE = "true";

  public static final String DB_FEATURE_PROPERTY_KEY = "astrum-db-feature";

  public static final String COMMUNITY = "astrum-community-edition";

  public void verify(final AstrumProperties astrumProperties) {
    // Ensure required properties exist
    requireProperty(astrumProperties, BASEDIR_SYS_PROP);
    requireProperty(astrumProperties, DATADIR_SYS_PROP);

    applyEnvironmentVariables(astrumProperties);

    // the select methods must happen after applying environment variables
    selectDatastoreFeature(astrumProperties);
    selectAuthenticationFeature(astrumProperties);

    requireProperty(astrumProperties, DB_FEATURE_PROPERTY_KEY);
    ensureHACIsDisabled(astrumProperties);
    selectDefaults(astrumProperties);
  }

  private static void applyEnvironmentVariables(final AstrumProperties astrumProperties) {
    maybeApplyEnvironmentVariable(
        astrumProperties,
        CHANGE_REPO_BLOBSTORE_TASK_ENABLED,
        "CHANGE_REPO_BLOBSTORE_TASK_ENABLED");
    maybeApplyEnvironmentVariable(astrumProperties, FIREWALL_QUARANTINE_FIX_ENABLED, "FIREWALL_QUARANTINE_FIX_ENABLED");
    maybeApplyEnvironmentVariable(astrumProperties, ZERO_DOWNTIME_BASELINE_FAIL, "ASTRUM_ZDU_BASELINE_FAIL");
    maybeApplyEnvironmentVariable(
        astrumProperties,
        ZERO_DOWNTIME_FUTURE_MIGRATION_ENABLED,
        "ASTRUM_ZDU_FUTURE_MIGRATION_ENABLED");
    maybeApplyEnvironmentVariable(astrumProperties, SECRETS_FILE, "ASTRUM_SECRETS_KEY_FILE");
    maybeApplyEnvironmentVariable(astrumProperties, DATASTORE_CLUSTERED_ENABLED, "DATASTORE_CLUSTERED_ENABLED");
    maybeApplyEnvironmentVariable(
        astrumProperties,
        CLUSTERED_ZERO_DOWNTIME_ENABLED,
        CLUSTERED_ZERO_DOWNTIME_ENABLED_ENV);
    maybeApplyEnvironmentVariable(astrumProperties, "astrum.clustered", "ASTRUM_CLUSTERED");
  }

  private static void selectDatastoreFeature(final AstrumProperties astrumProperties) {
    if (parseBoolean(astrumProperties.getProperty(DATASTORE_CLUSTERED_ENABLED))) {
      astrumProperties.put(SQL_DISTRIBUTED_CACHE, TRUE);
      astrumProperties.put(DATASTORE_BLOBSTORE_METRICS, TRUE);
    }
    else {
      // Set default of clustered upgrades off when clustering is off
      astrumProperties.put(CLUSTERED_ZERO_DOWNTIME_ENABLED, FALSE);
    }

    astrumProperties.put(DB_FEATURE_PROPERTY_KEY, "astrum-datastore-mybatis");
    astrumProperties.put(
        CHANGE_REPO_BLOBSTORE_TASK_ENABLED,
        astrumProperties.get(CHANGE_REPO_BLOBSTORE_TASK_ENABLED, TRUE));
    astrumProperties.put("astrum.quartz.jobstore.jdbc", TRUE);
  }

  private static void selectAuthenticationFeature(final AstrumProperties astrumProperties) {
    astrumProperties.put(ASTRUM_SECURITY_OAUTH2_ENABLED, TRUE);
    if (parseBoolean(astrumProperties.getProperty(DATASTORE_CLUSTERED_ENABLED))) {
      // if datastore is clustered, JWT must be enabled
      astrumProperties.put(JWT_ENABLED, TRUE);
      astrumProperties.put(SESSION_ENABLED, FALSE);
    }
    else if (isNullOrEmpty(astrumProperties.getProperty(SESSION_ENABLED))
        && isNullOrEmpty(astrumProperties.getProperty(JWT_ENABLED))) {
      astrumProperties.put(SESSION_ENABLED, TRUE);
      astrumProperties.put(JWT_ENABLED, FALSE);
    }
    else if (!isNullOrEmpty(astrumProperties.getProperty(SESSION_ENABLED))
        && isNullOrEmpty(astrumProperties.getProperty(JWT_ENABLED))) {
      astrumProperties.put(JWT_ENABLED, parseBoolean(astrumProperties.getProperty(SESSION_ENABLED)) ? FALSE : TRUE);
    }
    else if (!isNullOrEmpty(astrumProperties.getProperty(JWT_ENABLED))
        && isNullOrEmpty(astrumProperties.getProperty(SESSION_ENABLED))) {
      astrumProperties.put(SESSION_ENABLED, parseBoolean(astrumProperties.getProperty(JWT_ENABLED)) ? FALSE : TRUE);
    }

    if (!parseBoolean(astrumProperties.getProperty(JWT_ENABLED))) {
      // If JWT is not enabled, then disable OAuth2 as well
      astrumProperties.put(ASTRUM_SECURITY_OAUTH2_ENABLED, FALSE);
    }
  }

  private static void ensureHACIsDisabled(final AstrumProperties astrumProperties) {
    if (parseBoolean(astrumProperties.getProperty("astrum.clustered"))) {
      throw new IllegalStateException(
          "High Availability Clustering (HA-C) is a legacy feature and is no longer supported");
    }
  }

  private static void selectDefaults(final AstrumProperties astrumProperties) {
    if (astrumProperties.getProperty("astrum.onboarding.enabled") == null) {
      astrumProperties.put("astrum.onboarding.enabled", TRUE);
    }
    if (astrumProperties.getProperty("astrum.scripts.allowCreation") == null) {
      astrumProperties.put("astrum.scripts.allowCreation", FALSE);
    }
    if (astrumProperties.getProperty("astrum.http.denyframe.enabled") == null) {
      astrumProperties.put("astrum.http.denyframe.enabled", TRUE);
    }
  }

  private static void maybeApplyEnvironmentVariable(
      final AstrumProperties astrumProperties,
      final String propertyKey,
      final String environmentKey)
  {
    String envStr = System.getenv(environmentKey);

    if (envStr != null) {
      astrumProperties.put(propertyKey, envStr);
    }
  }

  private static void requireProperty(final AstrumProperties astrumProperties, final String name) {
    if (astrumProperties.getProperty(name) == null) {
      throw new IllegalStateException("Missing required property: " + name);
    }
  }
}
