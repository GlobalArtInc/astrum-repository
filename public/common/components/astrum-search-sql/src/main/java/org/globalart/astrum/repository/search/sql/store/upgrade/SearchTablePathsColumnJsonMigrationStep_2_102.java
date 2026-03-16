
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Changes the paths column type from VARCHAR to JSON in H2 database for the search_components table.
 */
@Component
public class SearchTablePathsColumnJsonMigrationStep_2_102
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String SEARCH_COMPONENTS_TABLE = "search_components";

  private static final String ALTER_PATHS_COLUMN_H2 =
      "ALTER TABLE search_components ALTER COLUMN paths SET DATA TYPE JSON";

  @Override
  public Optional<String> version() {
    return Optional.of("2.102");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, SEARCH_COMPONENTS_TABLE) && isH2(connection)) {
      log.info("Changing 'paths' column type from VARCHAR to JSON in table {}", SEARCH_COMPONENTS_TABLE);
      runStatement(connection, ALTER_PATHS_COLUMN_H2);
    }
    else {
      log.debug("Table {} doesn't exist or not running on H2, upgrade step {} will be skipped",
          SEARCH_COMPONENTS_TABLE, this.getClass().getSimpleName());
    }
  }
}
