
package org.globalart.astrum.repository.content.search.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.springframework.stereotype.Component;

/**
 * For SQL Search, change the column created to last_updated and trigger a re-index
 */
@Component
public class SqlSearchMigrationStep_1_29
    extends SearchIndexUpgrade
{
  private final String RENAME_QUERY = "ALTER TABLE search_components RENAME COLUMN component_created TO last_modified;";

  private final String NOT_NULL_QUERY = "ALTER TABLE search_components ALTER COLUMN last_modified DROP NOT NULL;";

  private final String TABLE = "search_components";

  private final String COMPONENT_CREATED = "component_created";

  private final String LAST_MODIFIED = "last_modified";

  @Override
  public Optional<String> version() {
    return Optional.of("1.29");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // We only need to do this for PostgreSQL, and the query will fail if the column exists
    if (isPostgresql(connection) && columnExists(connection, TABLE, COMPONENT_CREATED)) {
      log.info(String.format("Renaming column %s to %s", COMPONENT_CREATED, LAST_MODIFIED));
      runStatement(connection, RENAME_QUERY);

      log.info(String.format("Dropping null constraint"));
      runStatement(connection, NOT_NULL_QUERY);
    }

    super.migrate(connection);
  }
}
