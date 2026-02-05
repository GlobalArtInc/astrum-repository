
package org.globalart.astrum.repository.content.fluent.internal;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.FluentContinuation;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.fluent.constraints.FluentQueryConstraint;
import org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.types.GroupType;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.singletonList;
import static org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint.GroupRepositoryLocation.BOTH;
import static org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint.GroupRepositoryLocation.LOCAL;
import static org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint.GroupRepositoryLocation.MEMBERS;
import static org.globalart.astrum.repository.content.fluent.internal.RepositoryContentUtil.getRepositoryIds;
import static org.globalart.astrum.repository.content.fluent.internal.RepositoryContentUtil.isGroupRepository;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;
import static org.globalart.astrum.repository.content.store.InternalIds.toInternalId;

/**
 * {@link FluentAssets} implementation.
 *
 * @since 3.24
 */
public class FluentAssetsImpl
    implements FluentAssets
{
  private final ContentFacetSupport facet;

  private final AssetStore<?> assetStore;

  public FluentAssetsImpl(final ContentFacetSupport facet, final AssetStore<?> assetStore) {
    this.facet = checkNotNull(facet);
    this.assetStore = checkNotNull(assetStore);
  }

  @Override
  public FluentAssetBuilder path(final String path) {
    return new FluentAssetBuilderImpl(facet, assetStore, path);
  }

  @Override
  public FluentAsset with(final Asset asset) {
    return asset instanceof FluentAsset ? (FluentAsset) asset : new FluentAssetImpl(facet, asset);
  }

  @Override
  public int count() {
    return doCount(null, null, null);
  }

  int doCount(
      @Nullable final String kind,
      @Nullable final String filter,
      @Nullable final Map<String, Object> filterParams)
  {
    return assetStore.countAssets(facet.contentRepositoryId(), kind, filter, filterParams);
  }

  @Override
  public Continuation<FluentAsset> browse(final int limit, final String continuationToken) {
    List<FluentQueryConstraint> constraints = new ArrayList<>();
    if (isGroupRepository(facet.repository())) {
      constraints.add(new GroupRepositoryConstraint(LOCAL));
    }
    return doBrowse(limit, continuationToken, null, null, null, constraints);
  }

  /**
   * Browse assets with eagerly-loaded blobs and components, sorted by blob_created descending.
   *
   * <p>
   * <strong>NOTE:</strong> This method is currently only utilized by migration asset queries and polling operations
   * that require assets ordered by blob creation timestamp in descending order (newest first). It uses a composite
   * continuation token containing both blob_created timestamp and asset_id for stable pagination across pages.
   *
   * @see AssetBlobCreatedContinuationToken
   */
  @Override
  public Continuation<FluentAsset> browseEager(final int limit, @Nullable final String continuationToken) {
    return new FluentContinuation<>(assetStore.browseEagerAssets(facet.contentRepositoryId(), continuationToken, limit),
        this::with);
  }

  /**
   * Browse assets with eagerly-loaded blobs and components, sorted by blob_created descending,
   * with optional timestamp filtering.
   *
   * <p>
   * <strong>NOTE:</strong> This method is currently only utilized by migration asset queries and polling operations
   * that require assets ordered by blob creation timestamp in descending order (newest first). It uses a composite
   * continuation token containing both blob_created timestamp and asset_id for stable pagination across pages.
   *
   * @see AssetBlobCreatedContinuationToken
   */
  public Continuation<FluentAsset> browseEager(
      final int limit,
      @Nullable final String continuationToken,
      @Nullable final OffsetDateTime newerThan,
      @Nullable final OffsetDateTime olderThan)
  {
    return new FluentContinuation<>(
        assetStore.browseEagerAssets(facet.contentRepositoryId(), continuationToken, limit, newerThan, olderThan),
        this::with);
  }

  Continuation<FluentAsset> doBrowse(
      final int limit,
      @Nullable final String continuationToken,
      @Nullable final String kind,
      @Nullable final String filter,
      @Nullable final Map<String, Object> filterParams,
      @Nullable final List<FluentQueryConstraint> constraints)
  {
    Set<Integer> repositoryIds = getRepositoryIds(constraints, facet, facet.repository());

    return new FluentContinuation<>(assetStore.browseAssets(repositoryIds,
        continuationToken, kind, filter, filterParams, limit), this::with);
  }

  @Override
  public FluentQuery<FluentAsset> withGroupMemberContent() {
    return new FluentAssetQueryImpl(this, singletonList(new GroupRepositoryConstraint(BOTH)));
  }

  @Override
  public FluentQuery<FluentAsset> withOnlyGroupMemberContent() {
    return new FluentAssetQueryImpl(this, singletonList(new GroupRepositoryConstraint(MEMBERS)));
  }

  @Override
  public FluentQuery<FluentAsset> byKind(final String kind) {
    return new FluentAssetQueryImpl(this, kind);
  }

  @Override
  public FluentQuery<FluentAsset> byFilter(final String filter, final Map<String, Object> filterParams) {
    return new FluentAssetQueryImpl(this, filter, filterParams);
  }

  @Override
  public Optional<FluentAsset> find(final EntityId externalId) {
    return assetStore.readAsset(toInternalId(externalId))
        .filter(this::containedInRepository)
        .map(asset -> new FluentAssetImpl(facet, asset));
  }

  /**
   * Returns {@code true} if this asset is contained in this repository or any of its members.
   */
  private boolean containedInRepository(final Asset asset) {
    int expectedContentRepositoryId = contentRepositoryId(asset);
    if (expectedContentRepositoryId == facet.contentRepositoryId()) {
      return true;
    }
    else if (facet.repository().getType() instanceof GroupType) {
      return facet.repository()
          .facet(GroupFacet.class)
          .allMembers()
          .stream()
          .map(InternalIds::contentRepositoryId)
          .filter(Optional::isPresent)
          .map(Optional::get)
          .anyMatch(id -> id == expectedContentRepositoryId);
    }
    return false;
  }
}
