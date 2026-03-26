
package org.globalart.astrum.content.maven.internal.store;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.content.maven.store.GAV;
import org.globalart.astrum.content.maven.store.Maven2AssetBlobDAO;
import org.globalart.astrum.content.maven.store.Maven2AssetDAO;
import org.globalart.astrum.content.maven.store.Maven2ComponentDAO;
import org.globalart.astrum.content.maven.store.Maven2ComponentData;
import org.globalart.astrum.content.maven.store.Maven2ContentRepositoryDAO;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.store.BlobRefTypeHandler;
import org.globalart.astrum.repository.content.store.ContentRepositoryDAO;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.search.normalize.VersionNumberExpander;
import org.globalart.astrum.testdb.DataSessionRule;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static junit.framework.TestCase.assertTrue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.datastore.mybatis.CombUUID.combUUID;

public class Maven2ComponentDAOTest

{
  private ContentRepositoryData contentRepository;

  private int repositoryId;

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME)
      .handle(new BlobRefTypeHandler())
      .access(Maven2ContentRepositoryDAO.class)
      .access(Maven2ComponentDAO.class)
      .access(Maven2AssetBlobDAO.class)
      .access(Maven2AssetDAO.class);

  @Before
  public void setupContent() {
    contentRepository = new ContentRepositoryData();
    contentRepository.setConfigRepositoryId(new EntityUUID(combUUID()));
    contentRepository.setAttributes(newAttributes("repository"));

    createContentRepository(contentRepository);

    repositoryId = contentRepository.contentRepositoryId();

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ComponentDAO dao = session.access(Maven2ComponentDAO.class);
      generateComponent(1, "1.0.0", "1.0.0", dao);
      generateComponent(2, "1.0.0-20200914.113032-16", "1.0.0-SNAPSHOT", dao);
      generateComponent(3, "1.0.0-20201014.113032-17", "1.0.0-SNAPSHOT", dao);
      generateComponent(4, "1.0.0-20201114.113032-18", "1.0.0-SNAPSHOT", dao);
      session.getTransaction().commit();
    }
  }

  @Test
  public void findComponent() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ComponentDAO dao = session.access(Maven2ComponentDAO.class);
      Optional<Component> component = dao.readComponent(1);
      assertTrue(component.isPresent());
    }
  }

  @Test
  public void findGavsWithSnaphots() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ComponentDAO dao = session.access(Maven2ComponentDAO.class);
      Set<GAV> gavsWithSnaphots = dao.findGavsWithSnaphots(repositoryId, 2);
      assertThat(gavsWithSnaphots.size(), is(1));
      GAV gav = gavsWithSnaphots.stream().findFirst().get();
      assertThat(gav.group, is("group"));
      assertThat(gav.name, is("artifact"));
      assertThat(gav.baseVersion, is("1.0.0-SNAPSHOT"));
    }
  }

  @Test
  public void findGavsWithSnaphotsLessMinimum() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ComponentDAO dao = session.access(Maven2ComponentDAO.class);

      Set<GAV> gavsWithSnaphots = dao.findGavsWithSnaphots(repositoryId, 3);
      assertThat(gavsWithSnaphots.size(), is(0));
    }
  }

  @Test
  public void findComponentsForGav() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ComponentDAO dao = session.access(Maven2ComponentDAO.class);
      List<Maven2ComponentData> componentsForGav =
          dao.findComponentsForGav(repositoryId, "artifact", "group", "1.0.0-SNAPSHOT", "1.0.0");
      assertThat(componentsForGav.size(), is(3));
      List<String> componentVersions =
          componentsForGav.stream().map(component -> component.version()).collect(Collectors.toList());
      assertThat(componentVersions.contains("1.0.0-20200914.113032-16"), is(true));
      assertThat(componentVersions.contains("1.0.0-20201014.113032-17"), is(true));
      assertThat(componentVersions.contains("1.0.0-20201114.113032-18"), is(true));
    }
  }

  @Test
  public void selectSnapshotsAfterRelease() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ComponentDAO dao = session.access(Maven2ComponentDAO.class);
      int[] componentIds = dao.selectSnapshotsAfterRelease(repositoryId, 0);
      assertThat(componentIds.length, is(3));

      List<Integer> ids = Arrays.stream(componentIds).boxed().collect(Collectors.toList());
      assertThat(ids.contains(2), is(true));
      assertThat(ids.contains(3), is(true));
      assertThat(ids.contains(4), is(true));
    }
  }

  @Test
  public void selectSnapshotsAfterReleaseInGracePeriod() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      Maven2ComponentDAO dao = session.access(Maven2ComponentDAO.class);
      int[] componentIds = dao.selectSnapshotsAfterRelease(repositoryId, 1);
      assertThat(componentIds.length, is(0));
    }
  }

  private void createContentRepository(final ContentRepositoryData contentRepository) {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      ContentRepositoryDAO dao = session.access(Maven2ContentRepositoryDAO.class);
      dao.createContentRepository(contentRepository);
      session.getTransaction().commit();
    }
  }

  protected NestedAttributesMap newAttributes(final String key) {
    return new NestedAttributesMap("attributes", new HashMap<>(ImmutableMap.of(key, "test-value")));
  }

  private void generateComponent(
      final int componentId,
      final String version,
      final String baseVersion,
      final Maven2ComponentDAO dao)
  {
    Maven2ComponentData component = new Maven2ComponentData();
    component.setComponentId(componentId);
    component.setNamespace("group");
    component.setName("artifact");
    component.setVersion(version);
    component.setNormalizedVersion(VersionNumberExpander.expand(version));
    component.setBaseVersion(baseVersion);
    component.setAttributes(newAttributes("component"));
    component.setKind("aKind");
    component.setRepositoryId(repositoryId);
    dao.createComponent(component, false);
    dao.updateBaseVersion(component);
  }
}
