
package org.globalart.astrum.repository.content.search.upgrade;

import java.sql.Connection;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Recipe;
import org.globalart.astrum.repository.config.ConfigurationDAO;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.content.ContentRepository;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.example.TestContentRepositoryDAO;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.testdb.DataSessionRule;

import com.google.common.collect.ImmutableMap;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.mockito.junit.MockitoJUnitRunner;

import static java.lang.Boolean.TRUE;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.search.index.SearchUpdateService.SEARCH_INDEX_OUTDATED;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SearchIndexUpgradeTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME)
      .access(ConfigurationDAO.class)
      .access(TestContentRepositoryDAO.class);

  private DataStore<?> store;

  private ConfigurationDAO configurationDAO;

  private TestContentRepositoryDAO contentRepositoryDAO;

  @Mock
  private Recipe testHostedRecipe;

  @Mock
  private Recipe testProxyRecipe;

  @Mock
  private Recipe testGroupRecipe;

  private SearchIndexUpgrade underTest;

  private ConfigurationData hosted1Config;

  private ConfigurationData proxyConfig;

  private ConfigurationData groupConfig;

  private MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setup() {
    mockedStatic = mockStatic(QualifierUtil.class);
    Format testFormat = new Format("test")
    {
    };

    when(testHostedRecipe.getFormat()).thenReturn(testFormat);
    when(testHostedRecipe.getType()).thenReturn(new HostedType());

    when(testProxyRecipe.getFormat()).thenReturn(testFormat);
    when(testProxyRecipe.getType()).thenReturn(new ProxyType());

    when(testGroupRecipe.getFormat()).thenReturn(testFormat);
    when(testGroupRecipe.getType()).thenReturn(new GroupType());

    when(QualifierUtil.buildQualifierBeanMap(anyList())).thenReturn(ImmutableMap.of(
        "test-hosted", testHostedRecipe,
        "test-proxy", testProxyRecipe,
        "test-group", testGroupRecipe));
    underTest = new TestSearchIndexUpgrade();
    underTest.inject(List.of(testHostedRecipe, testProxyRecipe, testGroupRecipe));

    store = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME).get();
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      configurationDAO = session.access(ConfigurationDAO.class);
      contentRepositoryDAO = session.access(TestContentRepositoryDAO.class);

      hosted1Config = createConfig("hosted1", "test-hosted");
      createConfig("hosted2", "test-hosted");
      proxyConfig = createConfig("proxy", "test-proxy");
      groupConfig = createConfig("group", "test-group");

      createContentRepository(hosted1Config.getId(), new NestedAttributesMap());
      // intentionally not creating content repository for hosted2 to test mis-configuration
      NestedAttributesMap attrs = new NestedAttributesMap();
      attrs.child("test").set("value", "3");
      createContentRepository(proxyConfig.getId(), attrs);
      createContentRepository(groupConfig.getId(), new NestedAttributesMap());

      session.getTransaction().commit();
    }
  }

  @After
  public void teardown() {
    mockedStatic.close();
  }

  @Test
  public void testMigration() throws Exception {
    try (Connection conn = store.openConnection()) {
      underTest.migrate(conn);
    }

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      TestContentRepositoryDAO dao = session.access(TestContentRepositoryDAO.class);
      Optional<ContentRepository> foundHosted = dao.readContentRepository(hosted1Config.getId());
      Optional<ContentRepository> foundProxy = dao.readContentRepository(proxyConfig.getId());
      Optional<ContentRepository> foundGroup = dao.readContentRepository(groupConfig.getId());
      assertTrue(isRepositoryMarked(foundHosted.get()));
      assertTrue(isRepositoryMarked(foundProxy.get()));
      assertFalse(isRepositoryMarked(foundGroup.get()));

      // test that existing attributes are still there
      Map<String, Object> fooAttrs = (Map<String, Object>) foundProxy.get().attributes().get("test");
      assertEquals("3", fooAttrs.get("value"));
    }
  }

  private ConfigurationData createConfig(final String name, final String recipeName) {
    ConfigurationData config = new ConfigurationData();
    config.setName(name);
    config.setRecipeName(recipeName);
    config.setAttributes(ImmutableMap.of());

    configurationDAO.create(config);
    return config;
  }

  private void createContentRepository(final EntityId configId, final NestedAttributesMap attributes) {
    ContentRepositoryData repo = new ContentRepositoryData();
    repo.setConfigRepositoryId(configId);
    repo.setAttributes(attributes);

    contentRepositoryDAO.createContentRepository(repo);
  }

  private boolean isRepositoryMarked(final ContentRepository repo) {
    return TRUE.equals(repo.attributes().get(SEARCH_INDEX_OUTDATED));
  }

  private static class TestSearchIndexUpgrade
      extends SearchIndexUpgrade
  {
    @Override
    public Optional<String> version() {
      return Optional.of("1.0");
    }
  }
}
