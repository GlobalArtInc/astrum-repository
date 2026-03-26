
package org.globalart.astrum.content.maven.upgrade;

import java.sql.Connection;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.content.maven.store.Maven2ContentRepositoryDAO;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.repository.config.ConfigurationDAO;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.content.ContentRepository;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.testdb.DataSessionRule;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static java.lang.Boolean.TRUE;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.search.index.SearchUpdateService.SEARCH_INDEX_OUTDATED;

public class MavenIndexUpgradeTest

{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME)
      .access(ConfigurationDAO.class)
      .access(Maven2ContentRepositoryDAO.class);

  private DataStore<?> store;

  private ConfigurationDAO configurationDAO;

  private Maven2ContentRepositoryDAO contentRepositoryDAO;

  private final MavenIndexUpgrade underTest = new MavenIndexUpgrade()
  {
    @Override
    public Optional<String> version() {
      return Optional.of("1.0");
    }
  };

  private ConfigurationData hosted1Config;

  private ConfigurationData proxyConfig;

  private ConfigurationData groupConfig;

  @Before
  public void setup() {
    store = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME).get();
    try (DataSession session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      configurationDAO = (ConfigurationDAO) session.access(ConfigurationDAO.class);
      contentRepositoryDAO = (Maven2ContentRepositoryDAO) session.access(Maven2ContentRepositoryDAO.class);

      hosted1Config = createConfig("hosted1", "maven2-hosted");
      createConfig("hosted2", "maven2-hosted");
      proxyConfig = createConfig("proxy", "maven2-proxy");
      groupConfig = createConfig("group", "maven2-group");

      createContentRepository(hosted1Config.getId(), new NestedAttributesMap());
      // intentionally not creating content repository for hosted2 to test mis-configuration
      NestedAttributesMap attrs = new NestedAttributesMap();
      attrs.child("test").set("value", "3");
      createContentRepository(proxyConfig.getId(), attrs);
      createContentRepository(groupConfig.getId(), new NestedAttributesMap());

      session.getTransaction().commit();
    }
  }

  @Test
  public void testMigration() throws Exception {
    try (Connection conn = store.openConnection()) {
      underTest.migrate(conn);
    }

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ContentRepositoryDAO dao = session.access(Maven2ContentRepositoryDAO.class);
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
}
