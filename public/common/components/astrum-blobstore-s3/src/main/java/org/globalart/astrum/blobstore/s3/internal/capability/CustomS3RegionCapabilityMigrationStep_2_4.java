
package org.globalart.astrum.blobstore.s3.internal.capability;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

@Component
public class CustomS3RegionCapabilityMigrationStep_2_4
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.4");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op, this makes the CustomS3RegionCapability available
  }
}
