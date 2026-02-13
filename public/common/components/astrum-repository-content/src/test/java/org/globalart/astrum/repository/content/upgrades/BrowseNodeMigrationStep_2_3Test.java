
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.HashMap;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.repository.browse.node.RebuildBrowseNodesTaskDescriptor;
import org.globalart.astrum.repository.config.ConfigurationDAO;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.content.store.ContentRepositoryDAO;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.ExampleContentTestSupport;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.globalart.astrum.testdb.DatabaseTest;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static java.util.UUID.randomUUID;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@ExtendWith(MockitoExtension.class)
class BrowseNodeMigrationStep_2_3Test
    extends ExampleContentTestSupport
{
  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  @Mock
  private TaskConfiguration configuration;

  private DataStore<?> store;

  private BrowseNodeMigrationStep_2_3 underTest;

  @BeforeEach
  void setup() {
    sessionRule.register(ConanContentRepositoryDAO.class);
    lenient().when(taskScheduler.createTaskConfigurationInstance(RebuildBrowseNodesTaskDescriptor.TYPE_ID))
        .thenReturn(configuration);
    underTest = new BrowseNodeMigrationStep_2_3(taskScheduler, upgradeTaskScheduler);
    store = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME);
  }

  @DatabaseTest
  void testMigrationEmptyRepository() throws Exception {

    try (Connection conn = store.openConnection()) {
      underTest.migrate(conn);
    }
    verify(upgradeTaskScheduler, never()).schedule(configuration);
  }

  @DatabaseTest
  void testMigration() throws Exception {
    ContentRepositoryData contentRepositoryData = generateContentRepository();
    createRepository(contentRepositoryData);

    try (Connection conn = store.openConnection()) {
      underTest.migrate(conn);
    }
    verify(upgradeTaskScheduler, times(1)).schedule(configuration);

    cleanContentRepository(contentRepositoryData);
  }

  @DatabaseTest
  void testMigrationWithNonExistingTable() throws Exception {
    OssBrowseNodeMigrationStep_2_3 ossUnderTest =
        spy(new OssBrowseNodeMigrationStep_2_3(taskScheduler, upgradeTaskScheduler));
    try (Connection conn = store.openConnection()) {
      ossUnderTest.migrate(conn);
    }
    verify(ossUnderTest, never()).getRepositoryNames(any(Connection.class));
    verify(upgradeTaskScheduler, never()).schedule(configuration);
  }

  private void cleanContentRepository(final ContentRepositoryData contentRepositoryData) {
    try (DataSession<?> session = store.openSession()) {
      ContentRepositoryDAO dao = session.access(ConanContentRepositoryDAO.class);
      dao.deleteContentRepository(contentRepositoryData);
      session.getTransaction().commit();
    }
  }

  private ContentRepositoryData createRepository(
      final ContentRepositoryData contentRepository)
  {
    ConfigurationData entity = new ConfigurationData();
    entity.setRepositoryName("my-conan" + randomUUID());
    entity.setRecipeName("conan-hosted");
    entity.setOnline(true);
    entity.setAttributes(new HashMap<>());

    try (DataSession<?> session = store.openSession()) {
      ConfigurationDAO dao = session.access(ConfigurationDAO.class);
      dao.create(entity);
      session.getTransaction().commit();
    }

    contentRepository.setConfigRepositoryId(entity.getId());

    try (DataSession<?> session = store.openSession()) {
      ContentRepositoryDAO dao = session.access(ConanContentRepositoryDAO.class);
      dao.createContentRepository(contentRepository);
      session.getTransaction().commit();
    }

    return contentRepository;
  }

  private interface ConanContentRepositoryDAO
      extends ContentRepositoryDAO
  {
  }

  private class OssBrowseNodeMigrationStep_2_3
      extends BrowseNodeMigrationStep_2_3
  {
    OssBrowseNodeMigrationStep_2_3(final TaskScheduler taskScheduler, final UpgradeTaskScheduler upgradeTaskScheduler) {
      super(taskScheduler, upgradeTaskScheduler);
    }

    @Override
    public boolean tableExists(final Connection conn, final String tableName) {
      return false;
    }
  }
}
