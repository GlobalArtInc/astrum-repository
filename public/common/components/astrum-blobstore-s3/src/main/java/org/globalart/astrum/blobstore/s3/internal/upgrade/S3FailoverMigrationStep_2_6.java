
package org.globalart.astrum.blobstore.s3.internal.upgrade;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

@Component
public class S3FailoverMigrationStep_2_6
    implements DatabaseMigrationStep
{
  public static final String S3_FAILOVER_MIGRATION_VERSION = "2.6";

  @Override
  public Optional<String> version() {
    return Optional.of(S3_FAILOVER_MIGRATION_VERSION);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op, this makes the S3 failover region available
  }
}
