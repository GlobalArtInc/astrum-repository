
package org.globalart.astrum.internal.jwt.datastore.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Rule;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class JwtSessionAddTypeColumnMigrationStep_2_127_Test
{
  private static final String TABLE_NAME = "jwt_session";

  private static final String CREATE_TABLE_WITHOUT_TYPE = """
      CREATE TABLE IF NOT EXISTS jwt_session (
          user_session_id VARCHAR(36) NOT NULL,
          username        VARCHAR(200) NOT NULL,
          user_source     VARCHAR(200) NOT NULL,
          revoked_at      TIMESTAMP WITH TIME ZONE NOT NULL,
          expires_at      TIMESTAMP WITH TIME ZONE NOT NULL,
          CONSTRAINT pk_jwt_session_user_session_id PRIMARY KEY (user_session_id)
      )
      """;

  @Rule
  public DataSessionRule dataSessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME);

  private final JwtSessionAddTypeColumnMigrationStep_2_127 underTest =
      new JwtSessionAddTypeColumnMigrationStep_2_127();

  @Test
  public void testVersion() {
    assertEquals(Optional.of("2.127"), underTest.version());
  }

  @Test
  public void testMigrate_addsTypeColumn() throws Exception {
    try (Connection connection = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.runStatement(connection, CREATE_TABLE_WITHOUT_TYPE);
      assertFalse(underTest.columnExists(connection, TABLE_NAME, "type"));

      underTest.migrate(connection);

      assertTrue(underTest.columnExists(connection, TABLE_NAME, "type"));
    }
  }

  @Test
  public void testMigrate_isIdempotent() throws Exception {
    try (Connection connection = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.runStatement(connection, CREATE_TABLE_WITHOUT_TYPE);

      underTest.migrate(connection);
      underTest.migrate(connection);

      assertTrue(underTest.columnExists(connection, TABLE_NAME, "type"));
    }
  }

  @Test
  public void testMigrate_noErrorWhenTableDoesNotExist() throws Exception {
    try (Connection connection = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      assertFalse(underTest.tableExists(connection, TABLE_NAME));

      underTest.migrate(connection);

      assertFalse(underTest.tableExists(connection, TABLE_NAME));
    }
  }
}
