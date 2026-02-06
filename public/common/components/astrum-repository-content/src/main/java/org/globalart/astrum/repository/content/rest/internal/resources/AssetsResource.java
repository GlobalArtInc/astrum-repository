
package org.globalart.astrum.repository.content.rest.internal.resources;

import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.DetachedEntityId;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.MaintenanceService;
import org.globalart.astrum.repository.content.rest.internal.resources.doc.AssetsResourceDoc;
import org.globalart.astrum.repository.rest.api.AssetXO;
import org.globalart.astrum.repository.rest.api.AssetXODescriptor;
import org.globalart.astrum.repository.rest.api.RepositoryItemIDXO;
import org.globalart.astrum.repository.rest.api.RepositoryManagerRESTAdapter;
import org.globalart.astrum.repository.selector.ContentAuthHelper;
import org.globalart.astrum.rest.Page;
import org.globalart.astrum.rest.Resource;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.WebApplicationException;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authz.annotation.RequiresUser;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static java.lang.String.format;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.repository.content.rest.AssetXOBuilder.fromAsset;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetId;
import static org.globalart.astrum.repository.content.store.InternalIds.toExternalId;
import static org.globalart.astrum.repository.http.HttpStatus.UNPROCESSABLE_ENTITY;
import static org.globalart.astrum.repository.rest.api.RepositoryItemIDXO.fromString;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import static org.globalart.astrum.security.internal.uploadermetadata.UploaderMetadataSecurityContributor.UPLOADER_METADATA_READ_PERMISSION;

/**
 * @since 3.27
 */
@Component
@Path(AssetsResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public class AssetsResource
    extends AssetsResourceSupport
    implements Resource, AssetsResourceDoc
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/assets";

  private final RepositoryManagerRESTAdapter repositoryManagerRESTAdapter;

  private final MaintenanceService maintenanceService;

  private final Map<String, AssetXODescriptor> assetDescriptors;

  @Autowired
  public AssetsResource(
      final RepositoryManagerRESTAdapter repositoryManagerRESTAdapter,
      final MaintenanceService maintenanceService,
      final ContentAuthHelper contentAuthHelper,
      final List<AssetXODescriptor> assetDescriptorsList)
  {
    super(contentAuthHelper);
    this.repositoryManagerRESTAdapter = checkNotNull(repositoryManagerRESTAdapter);
    this.maintenanceService = checkNotNull(maintenanceService);
    this.assetDescriptors = QualifierUtil.buildQualifierBeanMap(assetDescriptorsList);
  }

  @RequiresUser
  @GET
  @Override
  public Page<AssetXO> getAssets(
      @Nullable @QueryParam("continuationToken") final String continuationToken,
      @QueryParam("repository") final String repositoryId)
  {
    Repository repository = repositoryManagerRESTAdapter.getRepository(repositoryId);
    List<FluentAsset> assets = browse(repository, continuationToken);
    return new Page<>(toAssetXOs(repository, assets, this.assetDescriptors), nextContinuationToken(assets));
  }

  @RequiresUser
  @GET
  @Path("/{id}")
  @Override
  public AssetXO getAssetById(@PathParam("id") final String id) {
    RepositoryItemIDXO repositoryItemIDXO = fromString(id);
    Repository repository = repositoryManagerRESTAdapter.getRepository(repositoryItemIDXO.getRepositoryId());
    Asset asset = getAsset(id, repository, new DetachedEntityId(repositoryItemIDXO.getId()));
    boolean uploaderVisible = SecurityUtils.getSubject().isPermitted(UPLOADER_METADATA_READ_PERMISSION);
    return fromAsset(asset, repository, this.assetDescriptors, uploaderVisible);
  }

  @RequiresUser
  @DELETE
  @Path("/{id}")
  @Override
  public void deleteAsset(@PathParam("id") final String id) {
    RepositoryItemIDXO repositoryItemIDXO = fromString(id);
    Repository repository = repositoryManagerRESTAdapter.getRepository(repositoryItemIDXO.getRepositoryId());

    DetachedEntityId entityId = new DetachedEntityId(repositoryItemIDXO.getId());
    Asset asset = getAsset(id, repository, entityId);
    maintenanceService.deleteAsset(repository, asset);
  }

  private Asset getAsset(final String id, final Repository repository, final DetachedEntityId entityId) {
    try {
      return repository.facet(ContentFacet.class)
          .assets()
          .find(entityId)
          .filter(assetPermitted(repository.getFormat().getValue(), repository.getName()))
          .orElseThrow(() -> new NotFoundException("Unable to locate asset with id " + id));
    }
    catch (IllegalArgumentException e) {
      log.debug("IllegalArgumentException caught retrieving asset with id {}", entityId, e);
      throw new WebApplicationException(format("Unable to process asset with id %s", entityId), UNPROCESSABLE_ENTITY);
    }
  }

  protected List<AssetXO> toAssetXOs(
      final Repository repository,
      final List<FluentAsset> assets,
      final Map<String, AssetXODescriptor> assetDescriptors)
  {
    boolean uploaderVisible = SecurityUtils.getSubject().isPermitted(UPLOADER_METADATA_READ_PERMISSION);
    return assets.stream()
        .map(asset -> fromAsset(asset, repository, assetDescriptors, uploaderVisible))
        .collect(toList());
  }

  private static String nextContinuationToken(final List<FluentAsset> assets) {
    int size = assets.size();
    return size < PAGE_SIZE_LIMIT ? null : toExternalId(internalAssetId(assets.get(size - 1))).getValue();
  }
}
