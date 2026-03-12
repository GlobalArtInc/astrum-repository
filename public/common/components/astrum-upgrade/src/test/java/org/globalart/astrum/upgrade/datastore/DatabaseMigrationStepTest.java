
package org.globalart.astrum.upgrade.datastore;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.content.testsuite.groups.PostgresTestGroup;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Rule;
import org.junit.Test;
import org.junit.experimental.categories.Category;

import static junit.framework.TestCase.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@Category(PostgresTestGroup.class)
public class DatabaseMigrationStepTest
{

  private static final String CUSTOM_SQL = "CREATE TABLE IF NOT EXISTS custom.test (\n"
      + "      domain            VARCHAR(200)   NOT NULL,\n"
      + "      token             VARCHAR(200)   NOT NULL,\n"
      + "      CONSTRAINT pk_domain PRIMARY KEY (domain)\n"
      + "    );\n"
      + "";

  @Rule
  public DataSessionRule customSessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME);

  @Test
  public void testCustomSchemaIndexExists() throws Exception {
    try (Connection conn = customSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.runStatement(conn, "drop schema if exists custom");
      underTest.runStatement(conn, "create schema custom authorization test");

      // create a table + index in custom schema
      underTest.runStatement(conn, CUSTOM_SQL);
      assertFalse("index does not exist in public schema", underTest.indexExists(conn, "pk_domain"));

      underTest.runStatement(conn, "SET search_path TO custom");
      assertTrue("index exists in custom schema", underTest.indexExists(conn, "pk_domain"));
    }
  }

  private final DatabaseMigrationStep underTest = new DatabaseMigrationStep()
  {
    public Optional<String> version() {
      return Optional.of("0.0");
    }

    @Override
    public void migrate(final Connection connection) {
    }
  };
}
