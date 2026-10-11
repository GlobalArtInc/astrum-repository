package org.globalart.astrum.repository.content.usage;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.stream.Collectors;
import javax.annotation.Nullable;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeData;
import org.globalart.astrum.repository.content.browse.store.example.TestBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.AssetBlobData;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ExampleContentTestSupport;
import org.globalart.astrum.repository.content.store.example.TestAssetBlobDAO;
import org.globalart.astrum.repository.content.store.example.TestAssetDAO;
import org.globalart.astrum.repository.content.store.example.TestComponentDAO;
import org.globalart.astrum.testdb.DatabaseTest;

import org.junit.jupiter.api.BeforeEach;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalComponentId;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;

/**
 * Test {@link UsageStore} against the tables of the {@code test} format.
 */
class UsageStoreTest
    extends ExampleContentTestSupport
{
  private static final OffsetDateTime NOW = OffsetDateTime.now();

  private UsageStore underTest;

  private int repositoryA;

  private int repositoryB;

  private ComponentData downloadedLongAgoButNewAsset;

  private ComponentData downloadedLately;

  private ComponentData neverDownloaded;

  private ComponentData neverDownloadedInB;

  private ComponentData downloadedLongAgo;

  private ComponentData withoutAssets;

  @BeforeEach
  void setUp() {
    sessionRule.register(TestBrowseNodeDAO.class, UsageDAO.class);
    underTest = new UsageStore(sessionRule);
    generateRepositories(2);
    repositoryA = generatedRepositories().get(0).contentRepositoryId();
    repositoryB = generatedRepositories().get(1).contentRepositoryId();

    downloadedLongAgoButNewAsset = component(repositoryA, "org", "c1");
    downloadedLately = component(repositoryA, "org", "c2");
    neverDownloaded = component(repositoryA, "", "c3");
    neverDownloadedInB = component(repositoryB, "org", "c4");
    downloadedLongAgo = component(repositoryA, "org", "c5");
    withoutAssets = component(repositoryA, "org", "c6");

    AssetData a1 = asset(repositoryA, downloadedLongAgoButNewAsset, "/org/c1/1.0/c1.jar", 100, "alice");
    asset(repositoryA, downloadedLongAgoButNewAsset, "/org/c1/1.0/c1.pom", 50, "bob");
    AssetData a3 = asset(repositoryA, downloadedLately, "/org/c2/1.0/c2.jar", 10, null);
    asset(repositoryA, neverDownloaded, "/c3/1.0/c3.jar", 7, null);
    asset(repositoryA, null, "/metadata.xml", 1000, null);
    AssetData a6 = asset(repositoryB, neverDownloadedInB, "/b/c4.jar", 5, null);
    AssetData a7 = asset(repositoryA, downloadedLongAgo, "/org/c5/1.0/c5.jar", 3, null);
    lastDownloaded(a1, NOW.minusDays(200));
    lastDownloaded(a3, NOW.minusDays(10));
    lastDownloaded(a7, NOW.minusDays(200));

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      TestBrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);
      BrowseNodeData org = node(repositoryA, 0, "org", "/org/", null, null);
      dao.mergeBrowseNode(org);
      BrowseNodeData c1 = node(repositoryA, org.getNodeId(), "c1", "/org/c1/", null, null);
      dao.mergeBrowseNode(c1);
      BrowseNodeData version = node(repositoryA, c1.getNodeId(), "1.0", "/org/c1/1.0/", downloadedLongAgoButNewAsset,
          null);
      dao.mergeBrowseNode(version);
      dao.mergeBrowseNode(node(repositoryA, version.getNodeId(), "c1.jar", "/org/c1/1.0/c1.jar",
          downloadedLongAgoButNewAsset, a1));
      BrowseNodeData b = node(repositoryB, 0, "b", "/b/", null, null);
      dao.mergeBrowseNode(b);
      dao.mergeBrowseNode(node(repositoryB, b.getNodeId(), "c4.jar", "/b/c4.jar", neverDownloadedInB, a6));
      session.getTransaction().commit();
    }
  }

  @DatabaseTest
  void countsTheAssetsAndAddsUpTheirSizesPerRepository() {
    Map<Integer, RepositoryContentUsage> usage = byRepository(underTest.browseContentUsage("test", NOW.minusDays(90)));

    assertThat(usage.get(repositoryA).getAssets(), is(6L));
    assertThat(usage.get(repositoryA).getBytes(), is(1170L));
    assertThat(usage.get(repositoryB).getAssets(), is(1L));
    assertThat(usage.get(repositoryB).getBytes(), is(5L));
  }

  @DatabaseTest
  void judgesTheLastDownloadOfAComponentLikeCleanupPolicies() {
    Map<Integer, RepositoryContentUsage> usage = byRepository(underTest.browseContentUsage("test", NOW.minusDays(90)));

    assertThat(usage.get(repositoryA).getUnusedComponents(), is(1L));
    assertThat(usage.get(repositoryA).getUnusedBytes(), is(3L));
    assertThat(usage.get(repositoryA).getNeverDownloadedComponents(), is(1L));
    assertThat(usage.get(repositoryA).getNeverDownloadedBytes(), is(7L));
    assertThat(usage.get(repositoryB).getUnusedComponents(), is(0L));
    assertThat(usage.get(repositoryB).getNeverDownloadedComponents(), is(1L));
    assertThat(usage.get(repositoryB).getNeverDownloadedBytes(), is(5L));
  }

  @DatabaseTest
  void countsComponentsAddedBeforeTheGivenTimeAsUnusedWhenTheyWereNeverDownloaded() {
    Map<Integer, RepositoryContentUsage> usage = byRepository(underTest.browseContentUsage("test", NOW.plusDays(1)));

    assertThat(usage.get(repositoryA).getUnusedComponents(), is(4L));
    assertThat(usage.get(repositoryA).getUnusedBytes(), is(170L));
    assertThat(usage.get(repositoryB).getUnusedComponents(), is(1L));
    assertThat(usage.get(repositoryB).getUnusedBytes(), is(5L));
  }

  @DatabaseTest
  void countsTheComponentsPerRepository() {
    Map<Integer, RepositoryContentUsage> components = byRepository(underTest.countComponents("test"));

    assertThat(components.get(repositoryA).getComponents(), is(5L));
    assertThat(components.get(repositoryB).getComponents(), is(1L));
  }

  @DatabaseTest
  void listsTheComponentsAddedLast() {
    assertThat(underTest.browseLatestComponentIds("test", null, 3),
        contains(id(withoutAssets), id(downloadedLongAgo), id(neverDownloadedInB)));
    assertThat(underTest.browseLatestComponentIds("test", List.of(repositoryA), 3),
        contains(id(withoutAssets), id(downloadedLongAgo), id(neverDownloaded)));
    assertThat(underTest.browseLatestComponentIds("test", List.of(), 3), is(empty()));
  }

  @DatabaseTest
  void readsComponentsWithTheirUploaderAndBrowseNode() {
    Map<Integer, ComponentSummary> components = underTest.browseComponents("test",
            List.of(id(downloadedLongAgoButNewAsset), id(neverDownloadedInB), id(neverDownloaded), 999_999))
        .stream()
        .collect(Collectors.toMap(ComponentSummary::getComponentId, Function.identity()));

    assertThat(components.size(), is(3));
    ComponentSummary c1 = components.get(id(downloadedLongAgoButNewAsset));
    assertThat(c1.getRepositoryId(), is(repositoryA));
    assertThat(c1.getNamespace(), is("org"));
    assertThat(c1.getName(), is("c1"));
    assertThat(c1.getVersion(), is("1.0"));
    assertThat(c1.getCreated(), is(notNullValue()));
    assertThat(c1.getCreatedBy(), is("alice"));
    assertThat(c1.getBrowsePath(), is("/org/c1/1.0/"));
    assertThat(components.get(id(neverDownloadedInB)).getBrowsePath(), is("/b/c4.jar"));
    assertThat(components.get(id(neverDownloaded)).getBrowsePath(), is(nullValue()));
    assertThat(components.get(id(neverDownloaded)).getCreatedBy(), is(nullValue()));
  }

  @DatabaseTest
  void acceptsOnlyFormatNames() {
    assertThrows(IllegalArgumentException.class, () -> underTest.countComponents("test_asset; DROP TABLE x --"));
    assertThrows(IllegalArgumentException.class, () -> underTest.browseComponents("te st", List.of(1)));
  }

  private ComponentData component(final int repositoryId, final String namespace, final String name) {
    ComponentData component = generateComponent(repositoryId, namespace, name, "1.0");
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(TestComponentDAO.class).createComponent(component, false);
      session.getTransaction().commit();
    }
    return component;
  }

  private AssetData asset(
      final int repositoryId,
      @Nullable final ComponentData component,
      final String path,
      final long size,
      @Nullable final String createdBy)
  {
    AssetData asset = generateAsset(repositoryId, path);
    asset.setComponent(component);
    AssetBlobData blob = assetBlob(NOW.minusHours(1));
    blob.setBlobSize(size);
    blob.setCreatedBy(createdBy);
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(TestAssetDAO.class).createAsset(asset, false);
      session.access(TestAssetBlobDAO.class).createAssetBlob(blob);
      asset.setAssetBlob(blob);
      session.access(TestAssetDAO.class).updateAssetBlobLink(asset, false);
      session.getTransaction().commit();
    }
    return asset;
  }

  private void lastDownloaded(final AssetData asset, final OffsetDateTime when) {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(TestAssetDAO.class).lastDownloaded(internalAssetId(asset), when);
      session.getTransaction().commit();
    }
  }

  private static BrowseNodeData node(
      final int repositoryId,
      final long parentId,
      final String displayName,
      final String requestPath,
      @Nullable final ComponentData component,
      @Nullable final AssetData asset)
  {
    BrowseNodeData node = new BrowseNodeData();
    node.setRepositoryId(repositoryId);
    node.setParentId(parentId);
    node.setDisplayName(displayName);
    node.setRequestPath(requestPath);
    node.setComponent(component);
    node.setAsset(asset);
    return node;
  }

  private static int id(final ComponentData component) {
    return internalComponentId(component);
  }

  private static Map<Integer, RepositoryContentUsage> byRepository(final List<RepositoryContentUsage> rows) {
    return rows.stream().collect(Collectors.toMap(RepositoryContentUsage::getRepositoryId, Function.identity()));
  }
}
