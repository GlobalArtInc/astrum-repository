
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.sql.Types;
import java.util.Collections;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.content.browse.RebuildBrowseNodesManager;
import org.globalart.astrum.repository.content.browse.store.example.TestBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ContentRepositoryDAO;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.ExampleContentTestSupport;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.content.store.example.TestContentRepositoryDAO;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.upgrade.datastore.UpgradeContext;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.db.api.Assertions.assertThat;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@ExtendWith(MockitoExtension.class)
class BrowseNodeMigrationStep_1_1Test
    extends ExampleContentTestSupport
{
  private final String INSERT = "INSERT INTO test_browse_node (repository_id, parent_id, display_name, request_path,"
      + " asset_id, component_id) "
      + " VALUES (?, ?, ?, ?, ?, ?);";

  @Mock
  private Format fakeFormat;

  @Mock
  private UpgradeContext upgradeContext;

  private BrowseNodeMigrationStep_1_1 upgradeStep;

  private DataStore<?> store;

  @BeforeEach
  void setup() {
    sessionRule.register(TestBrowseNodeDAO.class);
    when(fakeFormat.getValue()).thenReturn("test");
    upgradeStep = new BrowseNodeMigrationStep_1_1(Collections.singletonList(fakeFormat), upgradeContext);
    store = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME);
  }

  @DatabaseTest
  void testUnknownFormat() throws Exception {
    when(fakeFormat.getValue()).thenReturn("foo");
    BrowseNodeMigrationStep_1_1 upgradeStep =
        new BrowseNodeMigrationStep_1_1(Collections.singletonList(fakeFormat), upgradeContext);

    try (Connection conn = store.openConnection()) {
      upgradeStep.migrate(conn);
    }
  }

  @DatabaseTest
  void testMigration() throws Exception {
    generatePaths(10);
    generateNamespaces(10);
    generateNames(10);
    generateVersions(10);
    ContentRepositoryData repo = createContentRepository(generateContentRepository());

    insert(repo.contentRepositoryId(), 0, "jquery", "/jquery/", null, null);

    try (Connection conn = store.openConnection()) {
      upgradeStep.migrate(conn);
    }

    assertThat(sessionRule.table("test_browse_node")).isEmpty();

    verify(upgradeContext).setFlag(RebuildBrowseNodesManager.REBUILD_BROWSE_NODES_ON_START);
  }

  private void insert(
      final int repositoryId,
      final int parentId,
      final String displayName,
      final String requestPath,
      final AssetData asset,
      final ComponentData component) throws SQLException
  {
    // we don't use the dao because the index shouldn't exist for PostgreSQL at this phase
    try (Connection conn = store.openConnection()) {
      try (PreparedStatement statement = conn.prepareStatement(INSERT)) {
        statement.setInt(1, repositoryId);
        statement.setInt(2, parentId);
        statement.setString(3, displayName);
        statement.setString(4, requestPath);
        if (asset != null) {
          statement.setInt(5, InternalIds.internalAssetId(asset));
        }
        else {
          statement.setNull(5, Types.INTEGER);
        }
        if (component != null) {
          statement.setInt(6, InternalIds.internalComponentId(component));
        }
        else {
          statement.setObject(6, null);
        }
        statement.execute();
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
