
package org.globalart.astrum.coreui.internal.content;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.Date;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.WebApplicationException;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.coreui.AssetXO;
import org.globalart.astrum.coreui.ComponentHelper;
import org.globalart.astrum.coreui.ComponentXO;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.cache.CacheAttributeUtils;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.maintenance.MaintenanceService;
import org.globalart.astrum.repository.content.search.ComponentFinder;
import org.globalart.astrum.repository.content.security.AssetPermissionChecker;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.query.PageResult;
import org.globalart.astrum.repository.query.QueryOptions;
import org.globalart.astrum.repository.rest.api.AssetXODescriptor;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;
import org.globalart.astrum.repository.security.RepositorySelector;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.selector.CselSelector;
import org.globalart.astrum.selector.Selector;
import org.globalart.astrum.selector.SelectorFactory;
import org.globalart.astrum.selector.SelectorSqlBuilder;

import com.google.common.collect.ImmutableSet;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authz.AuthorizationException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.security.internal.uploadermetadata.UploaderMetadataSecurityContributor.UPLOADER_METADATA_READ_PERMISSION;
import static java.util.stream.Collectors.toList;
import static java.util.stream.Collectors.toSet;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import static org.globalart.astrum.repository.content.facet.ContentFacetFinder.findContentFacets;
import static org.globalart.astrum.repository.content.store.AssetDAO.FILTER_PARAMS;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalComponentId;
import static org.globalart.astrum.repository.content.store.InternalIds.toExternalId;
import static org.globalart.astrum.repository.view.Content.CONTENT;
import static org.globalart.astrum.repository.view.Content.CONTENT_LAST_MODIFIED;
import static org.globalart.astrum.security.BreadActions.BROWSE;

/**
 * Content-based {@link ComponentHelper}.
 *
 * @since 3.26
 */
@org.springframework.stereotype.Component
public class ContentComponentHelper
    implements ComponentHelper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final MaintenanceService maintenanceService;

  private final Map<String, ComponentFinder> componentFinders;

  private final ComponentFinder defaultComponentFinder;

  private final AssetPermissionChecker assetPermissionChecker;

  private final RepositoryManager repositoryManager;

  private final SelectorFactory selectorFactory;

  private final RepositoryPermissionChecker repositoryPermissionChecker;

  private final Map<String, AssetXODescriptor> assetDescriptors;

  @Autowired
  public ContentComponentHelper(
      final MaintenanceService maintenanceService,
      final List<ComponentFinder> componentFindersList,
      final AssetPermissionChecker assetPermissionChecker,
      final SelectorFactory selectorFactory,
      final RepositoryManager repositoryManager,
      final RepositoryPermissionChecker repositoryPermissionChecker,
      final List<AssetXODescriptor> assetDescriptorsList)
  {
    this.maintenanceService = checkNotNull(maintenanceService);
    this.componentFinders = QualifierUtil.buildQualifierBeanMap(checkNotNull(componentFindersList));
    this.assetPermissionChecker = checkNotNull(assetPermissionChecker);
    this.defaultComponentFinder =
        checkNotNull(componentFinders.get("default"));
    this.selectorFactory = checkNotNull(selectorFactory);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.repositoryPermissionChecker = checkNotNull(repositoryPermissionChecker);
    this.assetDescriptors = QualifierUtil.buildQualifierBeanMap(assetDescriptorsList);
  }

  @Override
  public List<AssetXO> readComponentAssets(final Repository repository, final ComponentXO model) {
    Optional<FluentComponent> component = findComponentsByModel(repository, model).findFirst();
    if (!component.isPresent()) {
      throw new WebApplicationException(NOT_FOUND);
    }

    Collection<FluentAsset> assets = component.get().assets();

    String format = repository.getFormat().getValue();

    boolean uploaderVisible = SecurityUtils.getSubject().isPermitted(UPLOADER_METADATA_READ_PERMISSION);
    return assetPermissionChecker.findPermittedAssets(assets, format, BROWSE)
        .map(entry -> {
          // Use the permitting repository as the repositoryName for the asset URL.
          // This ensures URLs use the group repo (e.g., maven-public) when user
          // accessed content through group permissions, not the member repo.
          String permittingRepository = entry.getValue();
          // Keep the actual repository name for the HostedType check which needs
          // to strip CONTENT_LAST_MODIFIED for proxied repos but keep it for hosted repos.
          // Cast to FluentAsset to access repository() method.
          FluentAsset fluentAsset = (FluentAsset) entry.getKey();
          String sourceRepository = fluentAsset.repository().getName();
          return toAssetXO(permittingRepository, permittingRepository, sourceRepository, format, entry.getKey(),
              uploaderVisible);
        })
        .collect(toList());
  }

  @Override
  @SuppressWarnings({"unchecked", "rawtypes"})
  public PageResult<AssetXO> previewAssets(
      final RepositorySelector repositorySelector,
      final List<Repository> selectedRepositories,
      final String jexlExpression,
      final QueryOptions queryOptions)
  {
    // Defence-in-depth authorization filter: re-run the browse check on the caller's input set
    // BEFORE group expansion. userCanBrowseRepositories tests direct RepositoryViewPermission, which
    // does not recognise group-mediated access — filtering after expansion drops leaves the caller
    // can only reach through a permitted parent group. Filtering pre-expansion preserves Astrum's
    // group permission model: a permitted group entitles preview of every leaf under it.
    List<Repository> permittedInputs =
        repositoryPermissionChecker.userCanBrowseRepositories(selectedRepositories);

    Set<Repository> previewRepositories = new LinkedHashSet<>();
    permittedInputs.forEach(r -> {
      if (r.getType() instanceof GroupType) {
        previewRepositories.addAll(r.facet(GroupFacet.class).leafMembers());
      }
      else {
        previewRepositories.add(r);
      }
    });

    int numAssets = 0;
    List<AssetXO> assets = new ArrayList<>();
    boolean uploaderVisible = SecurityUtils.getSubject().isPermitted(UPLOADER_METADATA_READ_PERMISSION);

    for (Repository r : previewRepositories) {
      String format = r.getFormat().getValue();

      SelectorSqlBuilder sqlBuilder = new SelectorSqlBuilder()
          .propertyAlias("path", "path")
          .propertyAlias("format", "'" + format + "'")
          .parameterPrefix("#{" + FILTER_PARAMS + ".")
          .parameterSuffix("}");

      Selector selector = selectorFactory.createSelector(CselSelector.TYPE, jexlExpression);
      selector.toSql(sqlBuilder);

      String filterString = sqlBuilder.getQueryString();
      Map<String, Object> filterParams = (Map) sqlBuilder.getQueryParameters();

      if (queryOptions.getFilter() != null) {
        filterString += " AND path LIKE #{" + FILTER_PARAMS + ".pathFilter}";
        filterParams.put("pathFilter", "%" + queryOptions.getFilter() + "%");
      }

      FluentQuery<FluentAsset> assetQuery = r.facet(ContentFacet.class)
          .assets()
          .byFilter(filterString, filterParams);

      numAssets += assetQuery.count();

      int nextLimit = queryOptions.getLimit() - assets.size();
      if (nextLimit > 0) {
        assetQuery.browse(nextLimit, null)
            .stream()
            .map(asset -> toAssetXO(r.getName(), r.getName(), r.getName(), format, asset, uploaderVisible))
            .collect(Collectors.toCollection(() -> assets));
      }
    }

    return new PageResult<>(numAssets, assets);
  }

  @Override
  public ComponentXO readComponent(final Repository repository, final EntityId componentId) {
    Optional<FluentComponent> component = findComponentById(repository, componentId);
    if (!component.isPresent()) {
      throw new WebApplicationException(NOT_FOUND);
    }

    Collection<FluentAsset> assets = component.get().assets();
    if (assets.isEmpty()) {
      throw new WebApplicationException(NOT_FOUND);
    }

    String repositoryName = repository.getName();
    String format = repository.getFormat().getValue();

    // display component if at least one asset is permitted
    if (assetPermissionChecker.findPermittedAssets(assets, format, BROWSE).findFirst().isPresent()) {
      return toComponentXO(repositoryName, format, component.get());
    }

    throw new AuthorizationException();
  }

  @Override
  public boolean canDeleteComponent(final Repository repository, final ComponentXO model) {
    return findComponentsByModel(repository, model)
        .allMatch(component -> maintenanceService.canDeleteComponent(repository, component));
  }

  @Override
  public Set<String> deleteComponent(final Repository repository, final ComponentXO model) {
    return findComponentsByModel(repository, model)
        .flatMap(component -> maintenanceService.deleteComponent(repository, component).stream())
        .collect(toSet());
  }

  @Override
  public AssetXO readAsset(final Repository repository, final EntityId assetId) {
    Optional<FluentAsset> asset = findAssetById(repository, assetId);
    if (!asset.isPresent()) {
      throw new WebApplicationException(NOT_FOUND);
    }

    String repositoryName = repository.getName();
    String format = repository.getFormat().getValue();

    boolean uploaderVisible = SecurityUtils.getSubject().isPermitted(UPLOADER_METADATA_READ_PERMISSION);
    return assetPermissionChecker.isPermitted(asset.get(), format, BROWSE)
        .map(containingRepositoryName -> {
          // Use the asset's actual repository for sourceRepository since that's where it lives.
          // Cast to FluentAsset to access repository() method.
          FluentAsset fluentAsset = (FluentAsset) asset.get();
          String sourceRepository = fluentAsset.repository().getName();
          return toAssetXO(repositoryName, containingRepositoryName, sourceRepository, format, asset.get(),
              uploaderVisible);
        })
        .orElseThrow(AuthorizationException::new);
  }

  @Override
  public boolean canDeleteAsset(final Repository repository, final EntityId assetId) {
    return findAssetById(repository, assetId)
        .filter(asset -> maintenanceService.canDeleteAsset(repository, asset))
        .isPresent();
  }

  @Override
  public Set<String> deleteAsset(final Repository repository, final EntityId assetId) {
    return findAssetById(repository, assetId)
        .map(asset -> maintenanceService.deleteAsset(repository, asset))
        .orElse(ImmutableSet.of());
  }

  @Override
  public boolean canDeleteFolder(final Repository repository, final String path) {
    return maintenanceService.canDeleteFolder(repository, path);
  }

  @Override
  public void deleteFolder(final Repository repository, final String path) {
    maintenanceService.deleteFolder(repository, path);
  }

  private Stream<FluentComponent> findComponentsByModel(final Repository repository, final ComponentXO model) {
    String format = repository.getFormat().getValue();
    ComponentFinder finder = componentFinders.getOrDefault(format, defaultComponentFinder);
    return finder.findComponentsByModel(repository,
        model.getId(),
        model.getGroup(),
        model.getName(),
        model.getVersion());
  }

  private Optional<FluentComponent> findComponentById(final Repository repository, final EntityId componentId) {
    return findContentFacets(repository)
        .map(ContentFacet::components)
        .map(facet -> facet.find(componentId))
        .filter(Optional::isPresent)
        .map(Optional::get)
        .findFirst();
  }

  private Optional<FluentAsset> findAssetById(final Repository repository, final EntityId assetId) {
    return findContentFacets(repository)
        .map(ContentFacet::assets)
        .map(facet -> facet.find(assetId))
        .filter(Optional::isPresent)
        .map(Optional::get)
        .findFirst();
  }

  @SuppressWarnings({"unchecked"})
  private ComponentXO toComponentXO(
      final String repositoryName,
      final String format,
      final Component component)
  {
    ComponentXO componentXO = new ComponentXO();
    componentXO.setRepositoryName(repositoryName);
    componentXO.setFormat(format);

    componentXO.setId(componentId(component));
    componentXO.setGroup(component.namespace());
    componentXO.setName(component.name());
    componentXO.setVersion(component.version());

    // Add component attributes similar to how assets handle them
    Map<String, Object> attributes = new HashMap<>(component.attributes().backing());
    Object formatAttributes = attributes.get(format);
    if (!Strings2.isEmpty(component.kind())) {
      if (formatAttributes instanceof Map) {
        ((Map<String, Object>) formatAttributes).put("component_kind", component.kind());
      }
      else {
        attributes.put(format, Collections.singletonMap("component_kind", component.kind()));
      }
    }

    // Filter format-specific attributes using AssetXODescriptor if available
    AssetXODescriptor descriptor = assetDescriptors.get(format);
    if (descriptor != null && formatAttributes instanceof Map) {
      Map<String, Object> originalFormatAttrs = (Map<String, Object>) formatAttributes;
      Map<String, Object> filteredFormatAttrs = new HashMap<>();

      Set<String> exposedKeys = descriptor.listExposedAttributeKeys();
      for (Map.Entry<String, Object> entry : originalFormatAttrs.entrySet()) {
        if (exposedKeys.contains(entry.getKey()) || "component_kind".equals(entry.getKey())) {
          filteredFormatAttrs.put(entry.getKey(), entry.getValue());
        }
      }

      attributes.put(format, filteredFormatAttrs);
    }

    componentXO.setAttributes(attributes);

    return componentXO;
  }

  @SuppressWarnings({"unchecked"})
  protected AssetXO toAssetXO(
      final String repositoryName,
      final String containingRepositoryName,
      final String sourceRepository,
      final String format,
      final Asset asset,
      final boolean uploaderVisible)
  {
    AssetXO assetXO = new AssetXO();
    assetXO.setRepositoryName(repositoryName);
    assetXO.setContainingRepositoryName(containingRepositoryName);
    assetXO.setFormat(format);

    assetXO.setId(assetId(asset));
    assetXO.setName(asset.path());

    asset.component().ifPresent(component -> assetXO.setComponentId(componentId(component)));

    Map<String, Object> attributes = new HashMap<>(asset.attributes().backing());
    Object formatAttributes = attributes.get(format);
    if (!Strings2.isEmpty(asset.kind())) {
      if (formatAttributes instanceof Map) {
        ((Map<String, Object>) formatAttributes).put("asset_kind", asset.kind());
      }
      else {
        attributes.put(format, Collections.singletonMap("asset_kind", asset.kind()));
      }
    }

    OffsetDateTime createdTime = asset.created();

    assetXO.setBlobCreated(Date.from(createdTime.toInstant()));
    asset.blob().ifPresent(blob -> {
      Date blobCreated = Date.from(blob.blobCreated().toInstant());

      // ASTRUM-31391 Asset created time may have been set incorrectly
      if (blob.blobCreated().isBefore(createdTime)) {
        assetXO.setBlobCreated(blobCreated);
      }

      assetXO.setBlobRef(blob.blobRef().toString());
      assetXO.setSize(blob.blobSize());
      assetXO.setContentType(blob.contentType());
      assetXO.setBlobUpdated(blobCreated);
      attributes.put("checksum", blob.checksums());
      if (uploaderVisible) {
        assetXO.setCreatedBy(blob.createdBy().orElse(null));
        assetXO.setCreatedByIp(blob.createdByIp().orElse(null));
      }
    });

    // Use the source repository for the HostedType check, not the permitting repository.
    // The source repository is where the asset actually lives; HostedType check determines
    // whether to strip CONTENT_LAST_MODIFIED (keep it for proxy repos, remove for hosted)
    if (repositoryManager.get(sourceRepository).getType() instanceof HostedType && attributes.containsKey(CONTENT)) {
      Map<String, Object> contentMap = (Map<String, Object>) attributes.get(CONTENT);
      contentMap.remove(CONTENT_LAST_MODIFIED);
    }

    // Filter format-specific attributes using AssetXODescriptor if available
    AssetXODescriptor descriptor = assetDescriptors.get(format);
    if (descriptor != null && formatAttributes instanceof Map) {
      Map<String, Object> originalFormatAttrs = (Map<String, Object>) formatAttributes;
      Map<String, Object> filteredFormatAttrs = new HashMap<>();

      Set<String> exposedKeys = descriptor.listExposedAttributeKeys();
      for (Map.Entry<String, Object> entry : originalFormatAttrs.entrySet()) {
        if (exposedKeys.contains(entry.getKey()) || "asset_kind".equals(entry.getKey())) {
          Object value = exposedKeys.contains(entry.getKey())
              ? descriptor.transformAttributeValue(entry.getKey(), entry.getValue())
              : entry.getValue();
          filteredFormatAttrs.put(entry.getKey(), value);
        }
      }

      attributes.put(format, filteredFormatAttrs);
    }

    assetXO.setAttributes(attributes);

    // Compute top-level registryUrl for pull snippets (docker only). Kept off attributes so it
    // does not appear in the Attributes panel (ASTRUM-51972).
    if (descriptor != null) {
      Repository repo = repositoryManager.get(repositoryName);
      String registryUrl = descriptor.computeRegistryUrl(repo);
      if (registryUrl != null) {
        assetXO.setRegistryUrl(registryUrl);
      }
    }

    asset.lastDownloaded().ifPresent(when -> assetXO.setLastDownloaded(Date.from(when.toInstant())));

    // Extract and set lastVerified from cache attributes (proxy repos only)
    Date lastVerified = CacheAttributeUtils.extractLastVerified(attributes);
    if (lastVerified != null) {
      assetXO.setLastVerified(lastVerified);
    }

    return assetXO;
  }

  private static String componentId(final Component component) {
    return toExternalId(internalComponentId(component)).getValue();
  }

  private static String assetId(final Asset asset) {
    return toExternalId(internalAssetId(asset)).getValue();
  }

}
