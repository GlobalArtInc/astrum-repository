
package org.globalart.astrum.repository.content.browse;

import java.math.BigDecimal;
import java.math.RoundingMode;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.concurrent.TimeUnit;
import java.util.function.Consumer;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.browse.node.BrowseNode;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeDAO;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeData;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeManager;
import org.globalart.astrum.repository.content.browse.store.BrowseNodeStore;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.store.FormatStoreManager;

import com.google.common.base.Stopwatch;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Scope;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static java.lang.Math.max;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;
import static org.globalart.astrum.repository.content.store.InternalIds.internalComponentId;
import static org.globalart.astrum.scheduling.CancelableHelper.checkCancellation;
import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * Default {@link BrowseFacet} implementation.
 *
 * @since 3.26
 */
@Scope(SCOPE_PROTOTYPE)
@org.springframework.stereotype.Component
public class BrowseFacetImpl
    extends FacetSupport
    implements BrowseFacet
{
  private static final int COMPONENT_ID_CACHE_SIZE = 10_000;

  // 0.75f is the DEFAULT_LOAD_FACTOR for LinkedHashMap
  private static final float CACHE_MAP_DEFAULT_LOAD_FACTOR = 0.75f;

  // Telling LinkedHashMap to retain order an act as an LRU
  private static final boolean CACHE_MAP_RETAIN_ACCESS_ORDER = true;

  private final Map<String, FormatStoreManager> formatStoreManagersByFormat;

  private final Map<String, BrowseNodeGenerator> browseNodeGeneratorsByFormat;

  private final int pageSize;

  private String format;

  private BrowseNodeGenerator browseNodeGenerator;

  private BrowseNodeManager browseNodeManager;

  @Autowired
  public BrowseFacetImpl(
      final List<FormatStoreManager> formatStoreManagersByFormatList,
      final List<BrowseNodeGenerator> browseNodeGeneratorsByFormatList,
      @Value("${astrum.browse.rebuild.pageSize:1000}") final int pageSize)
  {
    this.formatStoreManagersByFormat =
        QualifierUtil.buildQualifierBeanMap(checkNotNull(formatStoreManagersByFormatList));
    this.browseNodeGeneratorsByFormat =
        QualifierUtil.buildQualifierBeanMap(checkNotNull(browseNodeGeneratorsByFormatList));
    this.pageSize = max(pageSize, 1);
  }

  @Override
  protected void doStart() throws Exception {
    ContentFacetSupport contentFacet = (ContentFacetSupport) getRepository().facet(ContentFacet.class);

    format = getRepository().getFormat().getValue();
    String storeName = contentFacet.stores().contentStoreName;
    int repositoryId = contentFacet.contentRepositoryId();

    BrowseNodeStore<BrowseNodeDAO> browseNodeStore =
        lookupFormatStoreManager(format).formatStore(storeName, BrowseNodeDAO.class);

    browseNodeGenerator = lookupBrowseNodeGenerator(format);
    browseNodeManager = new BrowseNodeManager(browseNodeStore, repositoryId);
  }

  @Guarded(by = STARTED)
  @Override
  public List<BrowseNode> getByDisplayPath(
      final List<String> displayPath,
      final int limit,
      final String filter,
      final Map<String, Object> filterParams)
  {
    return browseNodeManager.getByDisplayPath(displayPath, limit, filter, filterParams);
  }

  @Guarded(by = STARTED)
  @Override
  public void addPathsToAssets(final Collection<EntityId> assetIds) {
    FluentAssets lookup = facet(ContentFacet.class).assets();

    Map<Integer, Integer> componentsProcessed = newComponentCache();

    assetIds.stream()
        .map(lookup::find)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(fluentAsset -> !browseNodeManager.hasAssetNode(fluentAsset))
        .forEach(fluentAsset -> createBrowseNodes(fluentAsset, componentsProcessed));
  }

  @Guarded(by = STARTED)
  @Override
  public void trimBrowseNodes() {
    browseNodeManager.trimBrowseNodes();
  }

  @Guarded(by = STARTED)
  @Override
  public void rebuildBrowseNodes(final Consumer<String> progressUpdater) {
    log.info("Deleting browse nodes for repository {}", getRepository().getName());

    browseNodeManager.deleteBrowseNodes();

    log.info("Rebuilding browse nodes for repository {}", getRepository().getName());

    createAllBrowseNodes(progressUpdater);
  }

  @Guarded(by = STARTED)
  @Override
  public void deleteByAssetIdAndPath(
      final Integer internalAssetId,
      final String path)
  {
    Long parentNodeId = deleteBrowseNodeByAsset(internalAssetId, path);

    if (parentNodeId != null) {
      List<BrowseNode> parentNodes = browseNodeManager.getNodeParents(parentNodeId);
      for (BrowseNode parentNode : parentNodes) {
        if (parentNode == null) {
          continue;
        }
        long assetCount = parentNode.getAssetCount() == null ? 0L : parentNode.getAssetCount();
        // Once a node has assets, we need to stop the node deletion
        if (assetCount > 0) {
          break;
        }

        BrowseNodeData nodeToDelete = (BrowseNodeData) parentNode;

        // If the node has no assets, check if it has children with assets or components
        boolean hasChildren = browseNodeManager.hasAnyAssetOrComponentChildren(nodeToDelete.getNodeId());
        if (hasChildren) {
          break;
        }

        log.debug("Deleting node with id = {} and path = '{}'", nodeToDelete.getNodeId(), nodeToDelete.getPath());
        deleteByNodeId(nodeToDelete.getNodeId());
      }
    }
  }

  @Override
  public void deleteByNodeId(final Long nodeId) {
    log.debug("Deleting browse node for repository = {} and node id = {}", getRepository().getName(), nodeId);
    browseNodeManager.delete(nodeId);
  }

  @Override
  public Optional<BrowseNode> getByRequestPath(final String requestPath) {
    return Optional.ofNullable(browseNodeManager.getByRequestPath(requestPath));
  }

  /**
   * Create browse nodes for every asset and their components in the repository.
   */
  private void createAllBrowseNodes(final Consumer<String> progressUpdater) {
    String repositoryName = getRepository().getName();
    try {
      FluentAssets assets = getRepository().facet(ContentFacet.class).assets();

      long total = assets.count();
      if (total > 0) {
        // useful for formats that have multiple assets per component
        Map<Integer, Integer> processedComponents = newComponentCache();
        ProgressLogIntervalHelper progressLogger = new ProgressLogIntervalHelper(log, 60);
        Stopwatch sw = Stopwatch.createStarted();

        long processed = 0;

        Continuation<FluentAsset> page = assets.browse(pageSize, null);
        while (!page.isEmpty()) {
          page.forEach(fluentAsset -> createBrowseNodes(fluentAsset, processedComponents));
          processed += page.size();

          long elapsed = sw.elapsed(TimeUnit.MILLISECONDS);
          progressLogger.info("Processed {} / {} {} assets in {} ms",
              processed, total, repositoryName, elapsed);
          if (progressUpdater != null) {
            long percentageComplete = BigDecimal.valueOf(processed)
                .divide(BigDecimal.valueOf(total),
                    2, RoundingMode.HALF_UP)
                .multiply(BigDecimal.valueOf(100))
                .longValue();
            progressUpdater.accept(
                String.format("%d%% Complete", percentageComplete));
          }

          checkCancellation();

          page = assets.browse(pageSize, page.nextContinuationToken());
        }

        progressLogger.flush(); // ensure the final progress message is flushed
      }
    }
    catch (Exception e) {
      log.error("Unable to rebuild browse nodes for repository {}", repositoryName, e);
    }
  }

  /**
   * Create browse nodes for an asset and it's component. Using a cache of component ids to limit component
   * nodes being recreated
   */
  private void createBrowseNodes(final FluentAsset asset, final Map<Integer, Integer> componentsProcessed) {
    if (browseNodeGenerator.hasMultipleAssetsPerComponent()) {
      createAssetBrowseNodes(asset);
      asset.component().ifPresent(component -> createComponentBrowseNodes(asset, component, componentsProcessed));
    }
    else {
      createCombinedAssetAndComponentBrowseNodes(asset);
    }
  }

  /**
   * Create browse nodes for each segment in an asset's path, assigning the asset to the final node
   */
  private void createAssetBrowseNodes(final FluentAsset asset) {
    List<BrowsePath> assetPaths = browseNodeGenerator.computeAssetPaths(asset);
    if (!assetPaths.isEmpty()) {
      browseNodeManager.createBrowseNodes(assetPaths, node -> node.setAsset(asset));
    }
  }

  /**
   * Create browse nodes for each segment in an asset's component's path, assigning the component to the final node,
   * if the asset has a component
   */
  private void createComponentBrowseNodes(
      final FluentAsset asset,
      final Component component,
      final Map<Integer, Integer> componentsProcessed)
  {
    Integer internalComponentId = internalComponentId(component);
    // null will be returned when adding a key that isn't already in the cache
    if (componentsProcessed.put(internalComponentId, internalComponentId) == null) {
      List<BrowsePath> componentPaths = browseNodeGenerator.computeComponentPaths(asset);
      if (!componentPaths.isEmpty()) {
        browseNodeManager.createBrowseNodes(componentPaths, node -> {
          node.setComponent(component);
        });
      }
    }
  }

  /**
   * Create browse nodes for each segment in the asset's path, and assign the asset and component to the final node
   */
  private void createCombinedAssetAndComponentBrowseNodes(final FluentAsset asset) {
    List<BrowsePath> assetPaths = browseNodeGenerator.computeAssetPaths(asset);
    if (!assetPaths.isEmpty()) {
      browseNodeManager.createBrowseNodes(assetPaths, node -> {
        node.setAsset(asset);
        asset.component().ifPresent(node::setComponent);
      });
    }
  }

  /**
   * Looks for the {@link FormatStoreManager} to use for the given repository format.
   */
  private FormatStoreManager lookupFormatStoreManager(final String format) {
    FormatStoreManager storeManager = formatStoreManagersByFormat.get(format);
    checkState(storeManager != null, "Could not find a store manager for format: %s", format);
    return storeManager;
  }

  /**
   * Looks for the {@link BrowseNodeGenerator} to use for the given repository format.
   */
  private BrowseNodeGenerator lookupBrowseNodeGenerator(final String format) {
    BrowseNodeGenerator generator = browseNodeGeneratorsByFormat.get(format);
    if (generator == null) {
      generator = browseNodeGeneratorsByFormat.get("default");
    }
    checkState(generator != null, "Could not find a browse node generator for format: %s", format);
    return generator;
  }

  private Map<Integer, Integer> newComponentCache() {
    return new LinkedHashMap<Integer, Integer>(COMPONENT_ID_CACHE_SIZE, CACHE_MAP_DEFAULT_LOAD_FACTOR,
        CACHE_MAP_RETAIN_ACCESS_ORDER)
    {
      @Override
      protected boolean removeEldestEntry(final Entry eldest) {
        return size() > COMPONENT_ID_CACHE_SIZE;
      }
    };
  }

  private Long deleteBrowseNodeByAsset(final Integer internalAssetId, final String path) {
    log.debug("Deleting browse nodes for repository = {} and asset path = {}", getRepository().getName(), path);
    Long parentNodeId = browseNodeManager.deleteByAssetIdAndPath(internalAssetId, path);
    if (parentNodeId == null) {
      // Delete only by its internal id
      parentNodeId = browseNodeManager.deleteByAssetId(internalAssetId);
    }
    log.debug("Deleted browse node for path = '{}' - Returned parent node {}", path, parentNodeId);
    return parentNodeId;
  }
}
