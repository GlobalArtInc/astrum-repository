
package org.globalart.astrum.repository.content.search.capability;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.content.testsuite.groups.PostgresTestGroup;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemDAO;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Recipe;
import org.globalart.astrum.repository.config.ConfigurationDAO;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.example.TestContentRepositoryDAO;
import org.globalart.astrum.testdb.DataSessionRule;

import org.assertj.core.api.Condition;
import org.assertj.core.condition.MappedCondition;
import org.assertj.db.type.Table;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.experimental.categories.Category;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.mockito.Mockito;
import org.mockito.junit.MockitoJUnitRunner;

import static org.assertj.core.condition.MappedCondition.mappedCondition;
import static org.assertj.db.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@Category(PostgresTestGroup.class)
@RunWith(MockitoJUnitRunner.Silent.class)
public class SearchCapabilityStep_2_16Test
{
  @Rule
  public DataSessionRule sessionRule =
      new DataSessionRule().access(CapabilityStorageItemDAO.class)
          .access(ConfigurationDAO.class)
          .access(TestContentRepositoryDAO.class);

  @Mock
  private Format format;

  @Mock
  private Recipe recipe;

  private MockedStatic<QualifierUtil> mockedStatic;

  private SearchCapabilityStep_2_16 underTest = new SearchCapabilityStep_2_16();

  @Before
  public void setup() {
    mockedStatic = Mockito.mockStatic(QualifierUtil.class);
    when(QualifierUtil.buildQualifierBeanMap(anyList())).thenReturn(Map.of("test-proxy", recipe));
    underTest.inject(List.of(recipe));
    lenient().when(recipe.getFormat()).thenReturn(format);
    lenient().when(format.getValue()).thenReturn("test");
  }

  @After
  public void tearDown() {
    mockedStatic.close();
  }

  @Test
  public void testMigrate_noCapability_noSearch() throws Exception {
    createRepository();
    try (Connection conn = spy(sessionRule.openConnection(DEFAULT_DATASTORE_NAME))) {
      underTest.migrate(conn);

      verify(conn, times(2)).prepareStatement("SELECT to_regclass(?);");
      // Check if table exists twice, plus a delete call to the table
      verify(conn, times(3)).prepareStatement(any());
    }
  }

  @Test
  public void testMigrate() throws Exception {
    stubSearchTable();
    createCapability();
    createRepository();
    try (Connection conn = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(conn);
    }

    assertThat(capabilityTable()).hasNumberOfRows(0);

    MappedCondition<byte[], String> condition =
        mappedCondition(b -> new String((byte[]) b),
            new Condition("{\"search_index_outdated\":true}"::equals, "equals"));

    assertThat(testContentRepositoryTable()).hasNumberOfRows(1)
        .row(0)
        .column("attributes")
        .value()
        .has(condition);
  }

  private void createRepository() {
    ConfigurationData configuration = new ConfigurationData();

    configuration.setAttributes(Map.of());
    configuration.setName("maven-central");
    configuration.setRecipeName("test-proxy");
    configuration.setRepositoryName("maven-central");

    ContentRepositoryData contentRepository = new ContentRepositoryData();
    contentRepository.setAttributes(new NestedAttributesMap());

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(ConfigurationDAO.class).create(configuration);

      contentRepository.setConfigRepositoryId(configuration.getRepositoryId());
      session.access(TestContentRepositoryDAO.class).createContentRepository(contentRepository);

      session.getTransaction().commit();
    }
  }

  private void createCapability() {
    CapabilityStorageItemData data = new CapabilityStorageItemData();
    data.setEnabled(true);
    data.setType("astrum.search.configuration");
    data.setNotes("");
    data.setProperties(Map.of());
    data.setVersion(1);
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(CapabilityStorageItemDAO.class).create(data);
      session.getTransaction().commit();
    }
  }

  private void stubSearchTable() {
    try (Connection conn = sessionRule.openConnection(DEFAULT_DATASTORE_NAME);
        PreparedStatement statement = conn.prepareStatement("CREATE TABLE search_components ( id int );")) {
      statement.execute();
    }
    catch (SQLException e) {
      throw new RuntimeException(e);
    }
  }

  private Table capabilityTable() {
    DataStore<?> dataStore = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME).orElseThrow(RuntimeException::new);
    return new Table(dataStore.getDataSource(), "capability_storage_item");
  }

  private Table testContentRepositoryTable() {
    DataStore<?> dataStore = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME).orElseThrow(RuntimeException::new);
    return new Table(dataStore.getDataSource(), "test_content_repository");
  }
}
