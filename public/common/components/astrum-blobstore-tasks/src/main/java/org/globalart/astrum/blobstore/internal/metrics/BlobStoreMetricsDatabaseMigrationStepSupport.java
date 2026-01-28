
package org.globalart.astrum.blobstore.internal.metrics;

import java.sql.Connection;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.common.BlobStoreTaskSupport;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.repository.content.blobstore.metrics.upgrade.UpgradeBlobStoreMetricsStore;
import org.globalart.astrum.repository.internal.blobstore.upgrade.UpgradeBlobStoreConfigurationStore;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Base task which will read the information from a metrics property file then adds it to the existing metrics total.
 *
 * This task should only be run manually for a particular blob store if it has previously failed, other usages may
 * result in double counting historical file based metric data.
 */
public abstract class BlobStoreMetricsDatabaseMigrationStepSupport
    extends StateGuardLifecycleSupport
    implements RepeatableDatabaseMigrationStep
{
  private final String blobStoreType;

  protected UpgradeBlobStoreMetricsStore metricsStore;

  protected UpgradeBlobStoreConfigurationStore blobStoreConfigurationStore;

  private UpgradeTaskScheduler upgradeTaskScheduler;

  protected BlobStoreMetricsDatabaseMigrationStepSupport(final String blobStoreType) {
    this.blobStoreType = checkNotNull(blobStoreType);
  }

  @Autowired
  public final void initDependencies(
      final UpgradeBlobStoreMetricsStore metricsStore,
      final UpgradeBlobStoreConfigurationStore blobStoreConfigurationStore,
      final UpgradeTaskScheduler upgradeTaskScheduler)
  {
    this.metricsStore = checkNotNull(metricsStore);
    this.blobStoreConfigurationStore = checkNotNull(blobStoreConfigurationStore);
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    log.info("Starting blob store metrics migration for type: {}", blobStoreType);

    String names = getBlobStoreConfigurations()
        .filter(this::shouldSaveMetricsToDatabase)
        .collect(Collectors.joining(","));

    if (names.isEmpty()) {
      log.debug("No blobstores of type {} found", blobStoreType);
      return;
    }

    TaskConfiguration configuration =
        upgradeTaskScheduler.createTaskConfigurationInstance(BlobStoreMetricsMigrationTask.TYPE_ID);

    configuration.setString(BlobStoreTaskSupport.BLOBSTORE_NAME_FIELD_ID, names);

    upgradeTaskScheduler.schedule(configuration);
  }

  /**
   * Implementations should override this if additional filtering should apply, e.g. skipping empty or zeroed metrics.
   */
  protected boolean shouldSaveMetricsToDatabase(final String blobStoreName) {
    return true;
  }

  protected Stream<String> getBlobStoreConfigurations() {
    return blobStoreConfigurationStore.list()
        .stream()
        .filter(store -> blobStoreType.equals(store.getType()))
        .map(BlobStoreConfiguration::getName);
  }
}
