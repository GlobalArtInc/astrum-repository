
package org.globalart.astrum.repository.content.browse.store;

import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.common.app.VersionComparator;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.repository.browse.node.BrowseNode;
import org.globalart.astrum.repository.browse.node.BrowseNodeComparator;
import org.globalart.astrum.repository.browse.node.DefaultBrowseNodeComparator;
import org.globalart.astrum.repository.content.browse.store.example.TestBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.AssetDAO;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ComponentDAO;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ContentRepositoryDAO;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.ExampleContentTestSupport;
import org.globalart.astrum.repository.content.store.example.TestAssetDAO;
import org.globalart.astrum.repository.content.store.example.TestComponentDAO;
import org.globalart.astrum.repository.content.store.example.TestContentRepositoryDAO;

import org.junit.jupiter.api.BeforeEach;
import org.globalart.astrum.testdb.DatabaseTest;

import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.greaterThan;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalComponentId;

/**
 * Test {@link BrowseNodeStore}.
 */
class BrowseNodeStoreTest
    extends ExampleContentTestSupport
{
  private static final BrowseNodeComparator byName = new DefaultBrowseNodeComparator(new VersionComparator());

  private ContentRepositoryData contentRepository;

  private ComponentData component1;

  private ComponentData component2;

  private AssetData asset1;

  private AssetData asset2;

  private BrowseNodeData alpha;

  private BrowseNodeData beta;

  private BrowseNodeData betaTwo;

  private BrowseNodeData betaThree;

  private BrowseNodeData gamma;

  private BrowseNodeData gammaOne;

  private BrowseNodeData gammaOneAlpha;

  private BrowseNodeStore<TestBrowseNodeDAO> browseNodeStore;

  @BeforeEach
  void setupContent() {
    sessionRule.register(TestBrowseNodeDAO.class);
    contentRepository = generateContentRepository();

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      ContentRepositoryDAO dao = session.access(TestContentRepositoryDAO.class);
      dao.createContentRepository(contentRepository);
      session.getTransaction().commit();
    }

    generateNamespaces(100);
    generateNames(100);
    generateVersions(100);
    generatePaths(100);

    component1 = generateComponent(1, "namespace1", "name1", "1.0.0");
    component2 = generateComponent(1, "namespace2", "name2", "2.0.0");
    component2.setVersion(component1.version() + "-2");

    asset1 = generateAsset(1, "/asset1/asset1.jar");
    asset2 = generateAsset(1, "/asset2/asset2.jar");
    asset2.setPath(asset1.path() + "/2");

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      ComponentDAO componentDao = session.access(TestComponentDAO.class);
      AssetDAO assetDao = session.access(TestAssetDAO.class);

      componentDao.createComponent(component1, false);
      componentDao.createComponent(component2, false);

      assetDao.createAsset(asset1, false);
      assetDao.createAsset(asset2, false);

      session.getTransaction().commit();
    }

    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);

      alpha = createNode(null, "alpha", "/a");
      dao.mergeBrowseNode(alpha);

      gamma = createNode(null, "gamma", "/g/");
      dao.mergeBrowseNode(gamma);

      beta = createNode(null, "beta", "/b/");
      dao.mergeBrowseNode(beta);

      gammaOne = createNode(gamma, "one", "/g/1/");
      gammaOne.dbComponentId = internalComponentId(component1);
      dao.mergeBrowseNode(gammaOne);

      betaTwo = createNode(beta, "two", "/b/2");
      betaTwo.dbComponentId = internalComponentId(component2);
      dao.mergeBrowseNode(betaTwo);

      gammaOneAlpha = createNode(gammaOne, "alpha", "/g/1/a");
      gammaOneAlpha.dbAssetId = internalAssetId(asset1);
      dao.mergeBrowseNode(gammaOneAlpha);

      betaThree = createNode(beta, "three", "/b/3");
      betaThree.dbAssetId = internalAssetId(asset2);
      dao.mergeBrowseNode(betaThree);

      session.getTransaction().commit();
    }

    browseNodeStore = new BrowseNodeStore<>(
        sessionRule, "astrum", TestBrowseNodeDAO.class);
  }

  @DatabaseTest
  void testDeleteBrowseNodesWithRangeBasedApproach() {
    // Verify we have nodes before deletion
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);
      Long maxNodeId = dao.getMaxNodeId(1);
      assertThat(maxNodeId, is(greaterThan(0L))); // Should have some nodes
    }

    // Use the store's deleteBrowseNodes method which uses our new range-based approach
    boolean deleted = browseNodeStore.deleteBrowseNodes(1);
    assertThat(deleted, is(true));

    // Verify all nodes are deleted
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);
      Long maxNodeIdAfter = dao.getMaxNodeId(1);
      assertThat(maxNodeIdAfter, is(nullValue())); // Should be null when no nodes exist
    }
  }

  @DatabaseTest
  void testDeleteBrowseNodesEmptyRepository() {
    // Delete from empty repository
    boolean deleted = browseNodeStore.deleteBrowseNodes(999);
    assertThat(deleted, is(false));
  }

  @DatabaseTest
  void testDeleteBrowseNodesWithLargeBatches() {
    // Create many more nodes to test batching behavior
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);

      for (int i = 0; i < 50; i++) {
        BrowseNodeData node = createNode(null, "node" + i, "/node" + i);
        dao.mergeBrowseNode(node);
      }
      session.getTransaction().commit();
    }

    // Verify we have many nodes
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);
      List<BrowseNode> listingBefore = getListing(dao);
      assertThat(listingBefore.size(), is(greaterThan(50)));
    }

    // Delete all using batched range-based approach
    boolean deleted = browseNodeStore.deleteBrowseNodes(1);
    assertThat(deleted, is(true));

    // Verify all nodes are deleted
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);
      List<BrowseNode> listingAfter = getListing(dao);
      assertThat(listingAfter.size(), is(0));
    }
  }

  @DatabaseTest
  void testDeleteBrowseNodesPartialDeletion() {
    // Create additional nodes with gaps in node_id sequence
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);

      // Delete some existing nodes to create gaps
      dao.deleteBrowseNodesByIdRange(1, 2, 3); // Delete middle nodes

      // Add new nodes
      for (int i = 0; i < 5; i++) {
        BrowseNodeData node = createNode(null, "newnode" + i, "/new" + i);
        dao.mergeBrowseNode(node);
      }
      session.getTransaction().commit();
    }

    // Verify store handles sparse node_id ranges correctly
    boolean deleted = browseNodeStore.deleteBrowseNodes(1);
    assertThat(deleted, is(true));

    // Verify all remaining nodes are deleted
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      BrowseNodeDAO dao = session.access(TestBrowseNodeDAO.class);
      List<BrowseNode> listingAfter = getListing(dao);
      assertThat(listingAfter.size(), is(0));
    }
  }

  private static List<BrowseNode> getListing(final BrowseNodeDAO dao, final String... paths) {
    List<BrowseNode> listing = dao.getByDisplayPath(1, asList(paths), 100, null, null);
    listing.sort(byName);
    return listing;
  }

  private static BrowseNodeData createNode(
      @Nullable final BrowseNodeData parent,
      final String displayName,
      final String requestPath)
  {
    BrowseNodeData node = new BrowseNodeData();
    node.setRepositoryId(1);
    node.setDisplayName(displayName);
    node.setRequestPath(requestPath);
    if (parent != null) {
      node.setParentId(parent.nodeId);
    }
    return node;
  }
}
