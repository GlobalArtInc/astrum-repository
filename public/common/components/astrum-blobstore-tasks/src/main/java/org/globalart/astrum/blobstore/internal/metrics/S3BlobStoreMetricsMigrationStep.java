
package org.globalart.astrum.blobstore.internal.metrics;

import java.io.IOException;
import java.sql.Connection;
import java.util.Collection;
import java.util.HashSet;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.kv.upgrade.UpgradeAstrumKeyValueStore;
import org.globalart.astrum.upgrade.datastore.UpgradeCooperation;

import com.fasterxml.jackson.databind.ObjectMapper;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;
import org.springframework.stereotype.Component;

/**
 * Migration step to move metrics from properties files in the blob store to the DB.
 *
 * For historical reasons we need to verify whether it was previously migrated.
 */
@ManagedLifecycle(phase = UPGRADE)
@Component
public class S3BlobStoreMetricsMigrationStep
    extends BlobStoreMetricsDatabaseMigrationStepSupport
{
  private static final String BLOB_STORE_TYPE = "S3";

  private static final String NAME = S3BlobStoreMetricsMigrationStep.class.getSimpleName();

  private final ObjectMapper objectMapper;

  private final UpgradeAstrumKeyValueStore kv;

  private final Cooperation2 cooperation;

  private Set<String> namesToMigrate;

  @Autowired
  public S3BlobStoreMetricsMigrationStep(
      final UpgradeAstrumKeyValueStore kv,
      final ObjectMapper objectMapper,
      final UpgradeCooperation upgradeCooperation)
  {
    super(BLOB_STORE_TYPE);
    this.kv = checkNotNull(kv);
    this.objectMapper = checkNotNull(objectMapper);
    this.cooperation = checkNotNull(upgradeCooperation).get(NAME);
  }

  @Override
  protected void doStart() throws IOException {
    // On clustered systems upgrades may not run at startup, so we need to determine what needs migrating before the
    // system might write to the DB.
    // Cooperation is used to avoid a race where a secondary node overwrites the first values
    cooperation.on(() -> {
      namesToMigrate = load();

      if (namesToMigrate == null) {
        Collection<String> s3BlobStores = compute();

        kv.setKey(new AstrumKeyValue(NAME, ValueType.OBJECT, s3BlobStores));
      }

      return namesToMigrate;
    })
        // Always check when the remote did the work just in case
        .checkFunction(() -> Optional.ofNullable(load()))
        .cooperate(NAME);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    super.migrate(connection);
    kv.removeKey(NAME);
  }

  @Override
  public Integer getChecksum() {
    return 0;
  }

  @Override
  protected boolean shouldSaveMetricsToDatabase(final String blobStoreName) {
    if (namesToMigrate == null) {
      namesToMigrate = compute();
    }
    return namesToMigrate.contains(blobStoreName);
  }

  private Set<String> load() {
    return kv.getKey(NAME)
        .map(val -> val.getAsObjectList(objectMapper, String.class))
        .map(HashSet::new)
        .orElse(null);
  }

  private Set<String> compute() {
    return getBlobStoreConfigurations()
        .filter(this::emptyOrWrong)
        .collect(Collectors.toSet());
  }

  private boolean emptyOrWrong(final String blobStoreName) {
    BlobStoreMetricsEntity metricsFromDb = metricsStore.get(blobStoreName);
    return metricsFromDb == null || metricsFromDb.getBlobCount() <= 0L;
  }
}
