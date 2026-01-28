
package org.globalart.astrum.blobstore.internal.metrics;

import java.util.List;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.blobstore.common.BlobStoreTaskSupport;
import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.kv.upgrade.UpgradeAstrumKeyValueStore;
import org.globalart.astrum.repository.content.blobstore.metrics.upgrade.UpgradeBlobStoreMetricsStore;
import org.globalart.astrum.repository.internal.blobstore.upgrade.UpgradeBlobStoreConfigurationStore;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.UpgradeCooperation;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests {@link S3BlobStoreMetricsMigrationStep}'s reworked, off-runtime-services wiring
 * ({@link UpgradeAstrumKeyValueStore} + {@link UpgradeCooperation} in place of the removed
 * {@code GlobalKeyValueStore} / {@code Cooperation2Factory}): it sources its {@link Cooperation2} from
 * {@link UpgradeCooperation}, schedules a single migration task only for S3 blob stores whose DB metrics are
 * absent or zeroed, and clears its KV bookkeeping key after migrating.
 */
@ExtendWith(MockitoExtension.class)
class S3BlobStoreMetricsMigrationStepTest
{
  private static final String NAME = S3BlobStoreMetricsMigrationStep.class.getSimpleName();

  private static final String S3 = "S3";

  @Mock
  private UpgradeAstrumKeyValueStore kv;

  @Mock
  private UpgradeCooperation upgradeCooperation;

  @Mock
  private Cooperation2 cooperation;

  @Mock
  private UpgradeBlobStoreMetricsStore metricsStore;

  @Mock
  private UpgradeBlobStoreConfigurationStore blobStoreConfigurationStore;

  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  @Mock
  private TaskConfiguration taskConfiguration;

  private S3BlobStoreMetricsMigrationStep underTest;

  @BeforeEach
  void setUp() {
    when(upgradeCooperation.get(NAME)).thenReturn(cooperation);
    underTest = new S3BlobStoreMetricsMigrationStep(kv, new ObjectMapper(), upgradeCooperation);
    underTest.initDependencies(metricsStore, blobStoreConfigurationStore, upgradeTaskScheduler);
  }

  @Test
  void cooperation_isObtainedFromUpgradeCooperation() {
    // The Cooperation2 instance must come from UpgradeCooperation.get(NAME) (the UPGRADE-safe wrapper),
    // not from the removed runtime Cooperation2Factory.
    verify(upgradeCooperation).get(NAME);
  }

  @Test
  void migrate_schedulesOnlyS3StoresWithAbsentOrZeroMetrics_thenClearsKvKey() throws Exception {
    BlobStoreConfiguration s3NoMetrics = config("s3-empty", S3);
    BlobStoreConfiguration s3WithMetrics = config("s3-populated", S3);
    BlobStoreConfiguration fileStore = config("file-bs", "File");
    when(blobStoreConfigurationStore.list()).thenReturn(List.of(s3NoMetrics, s3WithMetrics, fileStore));
    when(metricsStore.get("s3-empty")).thenReturn(null);
    when(metricsStore.get("s3-populated"))
        .thenReturn(new BlobStoreMetricsEntity().setBlobStoreName("s3-populated").setBlobCount(5L));
    when(upgradeTaskScheduler.createTaskConfigurationInstance(any())).thenReturn(taskConfiguration);

    underTest.migrate(null);

    // only the empty/zeroed S3 store is scheduled; the populated S3 store and the non-S3 store are excluded
    verify(taskConfiguration).setString(BlobStoreTaskSupport.BLOBSTORE_NAME_FIELD_ID, "s3-empty");
    verify(upgradeTaskScheduler).schedule(taskConfiguration);
    // migrate() clears the KV key it used to coordinate name computation across nodes
    verify(kv).removeKey(NAME);
  }

  @Test
  void migrate_noS3StoresNeedMigration_schedulesNothingButStillClearsKvKey() throws Exception {
    BlobStoreConfiguration s3WithMetrics = config("s3-populated", S3);
    when(blobStoreConfigurationStore.list()).thenReturn(List.of(s3WithMetrics));
    when(metricsStore.get("s3-populated"))
        .thenReturn(new BlobStoreMetricsEntity().setBlobStoreName("s3-populated").setBlobCount(10L));

    underTest.migrate(null);

    verify(upgradeTaskScheduler, never()).schedule(any());
    verify(kv).removeKey(NAME);
  }

  private static BlobStoreConfiguration config(final String name, final String type) {
    BlobStoreConfiguration configuration = mock(BlobStoreConfiguration.class);
    lenient().when(configuration.getName()).thenReturn(name);
    lenient().when(configuration.getType()).thenReturn(type);
    return configuration;
  }
}
