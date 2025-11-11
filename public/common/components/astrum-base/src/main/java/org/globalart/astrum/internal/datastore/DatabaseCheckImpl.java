
package org.globalart.astrum.internal.datastore;

import java.sql.Connection;
import java.util.Optional;
import java.util.concurrent.ExecutionException;

import org.springframework.beans.factory.annotation.Autowired;
import javax.sql.DataSource;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.common.upgrade.events.UpgradeEventSupport;
import org.globalart.astrum.datastore.api.DataStoreManager;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.cache.CacheBuilder;
import com.google.common.cache.CacheLoader;
import com.google.common.cache.LoadingCache;
import com.google.common.eventbus.Subscribe;
import org.flywaydb.core.Flyway;
import org.flywaydb.core.api.MigrationInfo;
import org.flywaydb.core.api.MigrationVersion;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.STORAGE;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import org.springframework.stereotype.Component;

@Component
@ManagedLifecycle(phase = STORAGE)
public class DatabaseCheckImpl
    extends StateGuardLifecycleSupport
    implements DatabaseCheck, EventAware
{
  private final LoadingCache<String, Boolean> cache = CacheBuilder.newBuilder().build(new AllowedCacheLoader());

  private final DataStoreManager dataStoreManager;

  private DataSource dataSource;

  private MigrationVersion currentSchemaVersion;

  private boolean postgresql = false;

  @Autowired
  public DatabaseCheckImpl(final DataStoreManager dataStoreManager) {
    this.dataStoreManager = checkNotNull(dataStoreManager);
  }

  @Override
  protected void doStart() throws Exception {
    dataSource = dataStoreManager.get(DEFAULT_DATASTORE_NAME)
        .orElseThrow(() -> new IllegalStateException("Missing DataStore named: " + DEFAULT_DATASTORE_NAME))
        .getDataSource();

    try (Connection con = dataSource.getConnection()) {
      postgresql = POSTGRE_SQL.equalsIgnoreCase(con.getMetaData().getDatabaseProductName());
    }
  }

  @Guarded(by = STARTED)
  @Override
  public boolean isPostgresql() {
    return postgresql;
  }

  @Override
  public boolean isAllowedByVersion(final Class<?> annotatedClass) {
    AvailabilityVersion availabilityVersion = annotatedClass.getAnnotation(AvailabilityVersion.class);
    if (availabilityVersion != null && isAllowed(availabilityVersion.from())) {
      return true;
    }
    if (availabilityVersion == null) {
      log.error("Missing database version specified for {}", annotatedClass);
    }

    log.debug("The database schema version is lower than the minimum required to enable {}", annotatedClass);
    return false;
  }

  @Override
  public boolean isAtLeast(final String version) {
    return isAllowed(version);
  }

  @Subscribe
  public void on(final UpgradeEventSupport event) {
    Optional<MigrationVersion> schemaVersion = event.getSchemaVersion()
        .map(MigrationVersion::fromVersion);
    if (schemaVersion.isPresent()) {
      currentSchemaVersion = schemaVersion.get();
    }
    cache.invalidateAll();
  }

  private boolean isAllowed(final String version) {
    try {
      return cache.get(version);
    }
    catch (ExecutionException e) {
      if (e.getCause() instanceof RuntimeException cause) {
        throw cause;
      }
      throw new RuntimeException(e); // NOSONAR
    }
  }

  @VisibleForTesting
  MigrationVersion getMigrationVersion() {
    if (dataSource == null) {
      log.warn("datasource has not been initialised");
      return null;
    }

    Flyway flyway = Flyway.configure()
        .dataSource(dataSource)
        .load();

    MigrationInfo current = flyway.info().current();
    if (current != null) {
      log.debug("Flyway info is reloaded, current version: {}", current.getVersion().getVersion());
      return current.getVersion();
    }

    log.error("Could not determine database schema version");
    return null;
  }

  private class AllowedCacheLoader
      extends CacheLoader<String, Boolean>
  {
    @Override
    public Boolean load(final String requiredVersion) {
      if (currentSchemaVersion == null || !currentSchemaVersion.isAtLeast(requiredVersion)) {
        currentSchemaVersion = getMigrationVersion();
        if (currentSchemaVersion == null) {
          return true;
        }
      }
      return currentSchemaVersion.isAtLeast(requiredVersion);
    }
  }
}
