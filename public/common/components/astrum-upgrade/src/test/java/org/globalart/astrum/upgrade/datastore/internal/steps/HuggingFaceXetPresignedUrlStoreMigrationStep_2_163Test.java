
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.sql.ResultSet;
import java.sql.Statement;
import java.util.Optional;

import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Integration tests for {@link HuggingFaceXetPresignedUrlStoreMigrationStep_2_163}. Exercises that the
 * {@code hf_xet_presigned_url_store} table is created with the expected columns and that the migration is
 * idempotent so re-runs after a partial failure don't explode.
 */
public class HuggingFaceXetPresignedUrlStoreMigrationStep_2_163Test
{
  private static final String TABLE_NAME = "hf_xet_presigned_url_store";

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME);

  private HuggingFaceXetPresignedUrlStoreMigrationStep_2_163 underTest;

  private DataStore<?> store;

  @Before
  public void setup() {
    underTest = new HuggingFaceXetPresignedUrlStoreMigrationStep_2_163();
    store = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME).get();
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();
    assertThat(version.isPresent(), is(true));
    assertThat(version.get(), is("2.163"));
  }

  @Test
  public void testMigrate_createsTable() throws Exception {
    try (Connection conn = store.openConnection()) {
      assertFalse("table should not exist yet", underTest.tableExists(conn, TABLE_NAME));

      underTest.migrate(conn);

      assertTrue("table should exist", underTest.tableExists(conn, TABLE_NAME));

      // Verify expected columns exist (H2 uppercases identifiers)
      assertTrue("token column should exist", columnExists(conn, TABLE_NAME, "TOKEN"));
      assertTrue("presigned_url column should exist", columnExists(conn, TABLE_NAME, "PRESIGNED_URL"));
      assertTrue("url_range column should exist", columnExists(conn, TABLE_NAME, "URL_RANGE"));
      assertTrue("expires_at_millis column should exist", columnExists(conn, TABLE_NAME, "EXPIRES_AT_MILLIS"));
      assertTrue("created_at_millis column should exist", columnExists(conn, TABLE_NAME, "CREATED_AT_MILLIS"));
    }
  }

  @Test
  public void testMigrate_isIdempotent() throws Exception {
    try (Connection conn = store.openConnection()) {
      underTest.migrate(conn);
      underTest.migrate(conn); // should not throw
      underTest.migrate(conn);

      assertTrue("table should still exist after multiple runs", underTest.tableExists(conn, TABLE_NAME));
    }
  }

  @Test
  public void testMigrate_canInsertAndReadBack() throws Exception {
    try (Connection conn = store.openConnection(); Statement stmt = conn.createStatement()) {
      underTest.migrate(conn);

      stmt.execute(
          "INSERT INTO hf_xet_presigned_url_store (token, presigned_url, url_range, expires_at_millis, created_at_millis) "
              + "VALUES ('test-token-123', 'https://example.com/presigned?sig=abc', 'bytes=0-1023', 9999999999, 1000000000)");

      try (ResultSet rs = stmt.executeQuery(
          "SELECT token, presigned_url, url_range, expires_at_millis FROM hf_xet_presigned_url_store WHERE token = 'test-token-123'")) {
        assertTrue("Should find inserted row", rs.next());
        assertThat(rs.getString("token"), is("test-token-123"));
        assertThat(rs.getString("presigned_url"), is("https://example.com/presigned?sig=abc"));
        assertThat(rs.getString("url_range"), is("bytes=0-1023"));
        assertThat(rs.getLong("expires_at_millis"), is(9999999999L));
      }
    }
  }

  @Test
  public void testMigrate_tokenIsPrimaryKey() throws Exception {
    try (Connection conn = store.openConnection(); Statement stmt = conn.createStatement()) {
      underTest.migrate(conn);

      stmt.execute(
          "INSERT INTO hf_xet_presigned_url_store (token, presigned_url, url_range, expires_at_millis, created_at_millis) "
              + "VALUES ('dup-token', 'https://example.com/1', 'bytes=0-100', 1000, 500)");

      // Same token again: PK violation
      boolean threw = false;
      try {
        stmt.execute(
            "INSERT INTO hf_xet_presigned_url_store (token, presigned_url, url_range, expires_at_millis, created_at_millis) "
                + "VALUES ('dup-token', 'https://example.com/2', 'bytes=100-200', 1000, 500)");
      }
      catch (Exception e) {
        threw = true;
      }
      assertTrue("Duplicate token insert should violate primary key", threw);
    }
  }

  private boolean columnExists(
      final Connection conn,
      final String tableName,
      final String columnName) throws Exception
  {
    String sql = String.format(
        "SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = '%s' AND COLUMN_NAME = '%s'",
        tableName.toUpperCase(), columnName.toUpperCase());

    try (Statement stmt = conn.createStatement(); ResultSet rs = stmt.executeQuery(sql)) {
      return rs.next() && rs.getInt(1) > 0;
    }
  }
}
