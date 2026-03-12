

package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class RemoveLog4JVisualizer_2_15
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String DELETE_CAPABILITY_ENTRIES = """
      DELETE FROM capability_storage_item
      WHERE type='log4j-visualizer';
      """;

  private static final String DROP_LOG_4J_TABLE = """
      DROP TABLE IF EXISTS log4j_visualizer;
      """;

  @Override
  public Optional<String> version() {
    return Optional.of("2.15");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    log.info("Dropping capabilities entries related to log4j visualizer");
    runStatement(connection, DELETE_CAPABILITY_ENTRIES);

    log.info("Dropping the log4j-visualizer table");
    runStatement(connection, DROP_LOG_4J_TABLE);
  }
}
