
package org.globalart.astrum.repository.content.maintenance.internal;

import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ExecutorService;
import jakarta.inject.Named;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.MissingFacetException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.content.maintenance.MaintenanceService;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.security.VariableResolverAdapterManager;
import org.globalart.astrum.selector.VariableSource;
import org.globalart.astrum.thread.AstrumThreadFactory;

import com.google.common.annotations.VisibleForTesting;
import org.apache.shiro.authz.AuthorizationException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static java.lang.Thread.MIN_PRIORITY;
import static java.time.OffsetDateTime.now;
import static java.util.concurrent.Executors.newSingleThreadExecutor;
import static org.globalart.astrum.security.BreadActions.DELETE;
import static org.globalart.astrum.thread.AstrumExecutorService.forCurrentSubject;

/**
 * @since 3.26
 */
@org.springframework.stereotype.Component
public class MaintenanceServiceImpl
    implements MaintenanceService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapterManager variableResolverAdapterManager;

  private final RepositoryPermissionChecker repositoryPermissionChecker;

  private final DeleteFolderService deleteFolderService;

  private final ExecutorService executorService;

  private final DatabaseCheck databaseCheck;

  @Autowired
  public MaintenanceServiceImpl(
      final ContentPermissionChecker contentPermissionChecker,
      final VariableResolverAdapterManager variableResolverAdapterManager,
      final RepositoryPermissionChecker repositoryPermissionChecker,
      final DeleteFolderService deleteFolderService,
      @Named("org.globalart.astrum.thread.DatabaseStatusDelayedExecutor") final ExecutorService executorService,
      final DatabaseCheck databaseCheck)
  {
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
    this.variableResolverAdapterManager = checkNotNull(variableResolverAdapterManager);
    this.repositoryPermissionChecker = checkNotNull(repositoryPermissionChecker);
    this.deleteFolderService = checkNotNull(deleteFolderService);
    this.databaseCheck = checkNotNull(databaseCheck);

    this.executorService = forCurrentSubject(newSingleThreadExecutor(
        new AstrumThreadFactory("delete-path", "Delete path in Tree Browse View", MIN_PRIORITY)));
  }

  @Override
  public Set<String> deleteAsset(final Repository repository, final Asset asset) {
    checkNotNull(repository);
    checkNotNull(asset);

    if (!canDeleteAsset(repository, asset)) {
      throw new AuthorizationException();
    }

    // Check for maintenance facet BEFORE deleting browse node to prevent orphaned browse nodes
    ContentMaintenanceFacet facet = maintenanceFacet(repository);
    deleteBrowseNode(repository, asset);

    return facet.deleteAsset(asset);
  }

  @Override
  public Set<String> deleteComponent(final Repository repository, final Component component) {
    checkNotNull(repository);
    checkNotNull(component);

    if (!canDeleteComponent(repository, component)) {
      throw new AuthorizationException();
    }

    // Check for maintenance facet BEFORE deleting browse node to prevent orphaned browse nodes
    ContentMaintenanceFacet facet = maintenanceFacet(repository);
    deleteBrowseNode(repository, component);

    return facet.deleteComponent(component);
  }

  @Override
  public void deleteFolder(final Repository repository, final String path) {
    checkNotNull(repository);
    checkNotNull(path);

    if (!canDeleteFolder(repository, path)) {
      throw new AuthorizationException();
    }

    executorService.submit(() -> deleteFolderService.deleteFolder(repository, path, now()));
  }

  @Override
  public boolean canDeleteComponent(final Repository repository, final Component component) {
    boolean canDeleteComponent = true;

    ContentFacet contentFacet = repository.facet(ContentFacet.class);

    String repositoryName = repository.getName();
    String format = repository.getFormat().getValue();

    VariableResolverAdapter variableResolverAdapter = variableResolverAdapterManager.get(format);

    for (Asset asset : contentFacet.components().with(component).assets()) {
      if (!canDeleteAssetInRepository(repositoryName, format, variableResolverAdapter, asset)) {
        canDeleteComponent = false;
        break;
      }
    }

    return canDeleteComponent;
  }

  @Override
  public boolean canDeleteAsset(final Repository repository, final Asset asset) {

    String repositoryName = repository.getName();
    String format = repository.getFormat().getValue();

    return canDeleteAssetInRepository(repositoryName, format, variableResolverAdapterManager.get(format), asset);
  }

  @Override
  public boolean canDeleteFolder(final Repository repository, final String folder) {
    // checking for repository-level permission
    if (!repositoryPermissionChecker.userCanDeleteInRepository(repository)) {
      return false;
    }
    // Evaluate Content Selector pattern against the folder path
    String repositoryName = repository.getName();
    String format = repository.getFormat().getValue();
    VariableResolverAdapter variableResolverAdapter = variableResolverAdapterManager.get(format);
    String folderPath = URLDecoder.decode(folder, StandardCharsets.UTF_8);
    if (!folderPath.startsWith("/")) {
      folderPath = "/" + folderPath;
    }
    VariableSource source = variableResolverAdapter.fromPath(folderPath, format);
    return contentPermissionChecker.isPermitted(repositoryName, format, DELETE, source);
  }

  @Override
  public Set<String> deleteAssets(final Repository repository, final List<Integer> assetIds) {
    checkNotNull(repository);
    checkNotNull(assetIds);
    if (assetIds.isEmpty()) {
      return Set.of();
    }

    AssetStore<?> assetStore = contentFacetSupport(repository).stores().assetStore;
    Set<String> deletedAssets = new HashSet<>();
    String repositoryName = repository.getName();
    for (Integer assetId : assetIds) {
      assetStore.readAsset(assetId).ifPresent(asset -> {
        Set<String> currentDeletedAssets = deleteAsset(repository, asset);
        log.trace("Current deleted assets {} from {} repository", currentDeletedAssets, repositoryName);
        deletedAssets.addAll(currentDeletedAssets);
      });
    }
    log.debug("Total deleted assets {} from {} repository", deletedAssets, repositoryName);
    return deletedAssets;
  }

  private boolean canDeleteAssetInRepository(
      final String repositoryName,
      final String format,
      final VariableResolverAdapter variableResolverAdapter,
      final Asset asset)
  {
    VariableSource source = variableResolverAdapter.fromPath(asset.path(), format);
    return contentPermissionChecker.isPermitted(repositoryName, format, DELETE, source);
  }

  private ContentMaintenanceFacet maintenanceFacet(final Repository repository) {
    try {
      return repository.facet(ContentMaintenanceFacet.class);
    }
    catch (MissingFacetException e) {
      throw new IllegalOperationException(format("Deleting from repository %s of type %s is not supported",
          repository.getName(), repository.getFormat()), e);
    }
  }

  protected void deleteBrowseNode(final Repository repository, final Asset asset) {
    if (isPostgresql()) {
      // For NPM repositories, only delete browse nodes for PACKAGE_ROOT assets during asset deletion
      // This preserves the UI tree structure when only metadata is being deleted
      if (isNpmRepository(repository) && shouldPreserveBrowseNode(asset)) {
        log.debug("Preserving browse node for NPM asset: {} (kind: {})", asset.path(), asset.kind());
        return;
      }

      Integer internalAssetId = InternalIds.internalAssetId(asset);

      repository.optionalFacet(BrowseFacet.class)
          .ifPresent(facet -> facet.deleteByAssetIdAndPath(internalAssetId, asset.path()));
    }
  }

  private boolean isNpmRepository(final Repository repository) {
    return "npm".equals(repository.getFormat().getValue());
  }

  private boolean shouldPreserveBrowseNode(final Asset asset) {
    // For NPM, preserve browse nodes for TARBALL assets (only delete for PACKAGE_ROOT)
    String assetKind = asset.kind();
    return "PACKAGE_ROOT".equals(assetKind);
  }

  private void deleteBrowseNode(final Repository repository, final Component component) {
    if (isPostgresql()) {
      repository.optionalFacet(ContentFacet.class)
          .ifPresent(contentFacet -> contentFacet.components()
              .with(component)
              .assets()
              .forEach(asset -> deleteBrowseNode(repository, asset)));
    }
  }

  private boolean isPostgresql() {
    return this.databaseCheck.isPostgresql();
  }

  @VisibleForTesting
  protected ContentFacetSupport contentFacetSupport(final Repository repository) {
    try {
      return (ContentFacetSupport) repository.facet(ContentFacet.class);
    }
    catch (Exception e) {
      throw new MissingFacetException(repository, ContentFacetSupport.class);
    }
  }
}
