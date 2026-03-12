
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class RemoveDistributedCooperationMigrationStep_2_14
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("2.14");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // with the decision to NOT completely remove distributed cooperation, we no longer need to do anything here
  }
}
