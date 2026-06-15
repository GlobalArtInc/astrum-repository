
package org.globalart.astrum.coreui.internal.blobstore;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.blobstore.s3.internal.upgrade.S3FailoverMigrationStep_2_6.S3_FAILOVER_MIGRATION_VERSION;
import static org.globalart.astrum.common.app.FeatureFlags.CLUSTERED_ZERO_DOWNTIME_ENABLED_NAMED_VALUE;
import org.springframework.stereotype.Component;

/**
 * State contributor to enable regional failover configuration for S3 blob stores.
 * The failover configuration will be available on ZDU if schema version is at least on version 2.6.
 */
@Component
public class S3FailoverStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final DatabaseCheck databaseCheck;

  private final boolean zduEnabled;

  @Autowired
  public S3FailoverStateContributor(
      final DatabaseCheck databaseCheck,
      @Value(CLUSTERED_ZERO_DOWNTIME_ENABLED_NAMED_VALUE) final boolean zduEnabled)
  {
    this.databaseCheck = databaseCheck;
    this.zduEnabled = zduEnabled;
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of("S3FailoverEnabled", isAvailable());
  }

  private boolean isAvailable() {
    return !zduEnabled || databaseCheck.isAtLeast(S3_FAILOVER_MIGRATION_VERSION);
  }
}
