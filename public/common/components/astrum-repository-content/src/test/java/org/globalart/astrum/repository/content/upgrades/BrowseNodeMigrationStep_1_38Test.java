
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.Arrays;
import java.util.Collections;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.content.browse.store.example.TestBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.ContentRepositoryDAO;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.ExampleContentTestSupport;
import org.globalart.astrum.repository.content.store.example.TestContentRepositoryDAO;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.globalart.astrum.testdb.DatabaseTest;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@ExtendWith(MockitoExtension.class)
class BrowseNodeMigrationStep_1_38Test
    extends ExampleContentTestSupport
{
  private final String INSERT = "INSERT INTO test_browse_node (repository_id, parent_id, display_name, request_path)"
      + " VALUES (?, ?, ?, ?);";

  @Mock
  private Format testFormat;

  @Mock
  private Format maven2Format;

  @Mock
  private Format npmFormat;

  private BrowseNodeMigrationStep_1_38 upgradeStep;

  private DataStore<?> store;

  @BeforeEach
  void setup() {
    sessionRule.register(TestBrowseNodeDAO.class);
    lenient().when(testFormat.getValue()).thenReturn("test");
    lenient().when(maven2Format.getValue()).thenReturn("maven2");
    lenient().when(npmFormat.getValue()).thenReturn("npm");
    upgradeStep = new BrowseNodeMigrationStep_1_38(Arrays.asList(testFormat, maven2Format, npmFormat));
    store = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME);
  }

  @DatabaseTest
  void testIndexCreation() throws Exception {
    ContentRepositoryData repo = createContentRepository(generateContentRepository());
    insertBrowseNode(repo.contentRepositoryId(), 0, "jquery", "/jquery/");

    assertFalse(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Index should not exist before migration");

    try (Connection conn = store.openConnection()) {
      upgradeStep.migrate(conn);
    }

    assertTrue(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Index should exist after migration");
  }

  @DatabaseTest
  void testIdempotentMigration() throws Exception {
    ContentRepositoryData repo = createContentRepository(generateContentRepository());
    insertBrowseNode(repo.contentRepositoryId(), 0, "jquery", "/jquery/");

    try (Connection conn = store.openConnection()) {
      upgradeStep.migrate(conn);
    }

    assertTrue(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Index should exist after first migration");

    try (Connection conn = store.openConnection()) {
      upgradeStep.migrate(conn);
    }

    assertTrue(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Index should still exist after second migration");
  }

  @DatabaseTest
  void testNonExistentTable() throws Exception {
    when(testFormat.getValue()).thenReturn("nonexistent");
    BrowseNodeMigrationStep_1_38 step = new BrowseNodeMigrationStep_1_38(Collections.singletonList(testFormat));

    try (Connection conn = store.openConnection()) {
      step.migrate(conn);
    }

    assertFalse(indexExists("nonexistent_browse_node", "idx_nonexistent_browse_node_parent_id"),
        "Index should not be created for non-existent table");
  }

  @DatabaseTest
  void testMultipleFormats() throws Exception {
    ContentRepositoryData repo = createContentRepository(generateContentRepository());
    insertBrowseNode(repo.contentRepositoryId(), 0, "jquery", "/jquery/");

    assertFalse(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Test index should not exist before migration");

    try (Connection conn = store.openConnection()) {
      upgradeStep.migrate(conn);
    }

    assertTrue(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Test index should exist after migration");
  }

  @DatabaseTest
  void testEmptyTable() throws Exception {
    createContentRepository(generateContentRepository());

    assertFalse(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Index should not exist before migration");

    try (Connection conn = store.openConnection()) {
      upgradeStep.migrate(conn);
    }

    assertTrue(indexExists("test_browse_node", "idx_test_browse_node_parent_id"),
        "Index should be created even on empty table");
  }

  private void insertBrowseNode(
      final int repositoryId,
      final int parentId,
      final String displayName,
      final String requestPath) throws SQLException
  {
    try (Connection conn = store.openConnection()) {
      try (PreparedStatement statement = conn.prepareStatement(INSERT)) {
        statement.setInt(1, repositoryId);
        statement.setInt(2, parentId);
        statement.setString(3, displayName);
        statement.setString(4, requestPath);
        statement.execute();
      }
    }
  }

  private boolean indexExists(final String tableName, final String indexName) throws SQLException {
    try (Connection conn = store.openConnection()) {
      if (conn.getMetaData().getDatabaseProductName().equals("H2")) {
        String sql =
            "SELECT * FROM INFORMATION_SCHEMA.INDEXES WHERE UPPER(TABLE_NAME) = ? AND UPPER(INDEX_NAME) LIKE ?";
        try (PreparedStatement stmt = conn.prepareStatement(sql)) {
          stmt.setString(1, tableName.toUpperCase());
          stmt.setString(2, indexName.toUpperCase() + "%");
          try (ResultSet rs = stmt.executeQuery()) {
            return rs.next();
          }
        }
      }
      else {
        String sql = "SELECT * FROM pg_indexes WHERE UPPER(tablename) = ? AND UPPER(indexname) = ?";
        try (PreparedStatement stmt = conn.prepareStatement(sql)) {
          stmt.setString(1, tableName.toUpperCase());
          stmt.setString(2, indexName.toUpperCase());
          try (ResultSet rs = stmt.executeQuery()) {
            return rs.next();
          }
        }
      }
    }
  }

  private ContentRepositoryData createContentRepository(final ContentRepositoryData contentRepository) {
    try (DataSession<?> session = store.openSession()) {
      ContentRepositoryDAO dao = session.access(TestContentRepositoryDAO.class);
      dao.createContentRepository(contentRepository);
      session.getTransaction().commit();
    }
    return contentRepository;
  }
}
