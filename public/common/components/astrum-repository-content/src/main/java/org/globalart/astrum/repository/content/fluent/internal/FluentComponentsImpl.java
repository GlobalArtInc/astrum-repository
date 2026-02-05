
package org.globalart.astrum.repository.content.fluent.internal;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.function.Function;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.ComponentSet;
import org.globalart.astrum.repository.content.SqlGenerator;
import org.globalart.astrum.repository.content.SqlQueryParameters;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponentBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.fluent.FluentContinuation;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.fluent.constraints.FluentQueryConstraint;
import org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ComponentSetData;
import org.globalart.astrum.repository.content.store.ComponentStore;
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
 * {@link FluentComponents} implementation.
 *
 * @since 3.24
 */
public class FluentComponentsImpl
    implements FluentComponents
{
  private final ContentFacetSupport facet;

  private final ComponentStore<?> componentStore;

  private final Function<String, String> versionNormalizer;

  public FluentComponentsImpl(
      final ContentFacetSupport facet,
      final ComponentStore<?> componentStore,
      final Function<String, String> versionNormalizer)
  {
    this.facet = checkNotNull(facet);
    this.componentStore = checkNotNull(componentStore);
    this.versionNormalizer = checkNotNull(versionNormalizer);
  }

  @Override
  public FluentComponentBuilder name(final String name) {
    return new FluentComponentBuilderImpl(facet, componentStore, versionNormalizer, name);
  }

  @Override
  public FluentComponent with(final Component component) {
    return component instanceof FluentComponent
        ? (FluentComponent) component
        : new FluentComponentImpl(facet, component);
  }

  @Override
  public FluentComponent with(final Component component, @Nullable final Collection<Asset> assets) {
    if (component instanceof FluentComponent) {
      return (FluentComponent) component;
    }

    if (assets == null) {
      return new FluentComponentImpl(facet, component, null);
    }

    List<FluentAsset> fluentAssets = assets
        .stream()
        .map(it -> facet.assets().with(it))
        .collect(Collectors.toList());

    return new FluentComponentImpl(facet, component, fluentAssets);
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
    if (isNugetV2Proxy()) {
      return componentStore.countComponentsWithAssetsBlobs(facet.contentRepositoryId(), kind, filter, filterParams);
    }
    return componentStore.countComponents(facet.contentRepositoryId(), kind, filter, filterParams);
  }

  @Override
  public Continuation<FluentComponent> browse(final int limit, final String continuationToken) {
    List<FluentQueryConstraint> constraints = new ArrayList<>();
    if (isGroupRepository(facet.repository())) {
      constraints.add(new GroupRepositoryConstraint(LOCAL));
    }
    return doBrowse(limit, continuationToken, null, null, null, constraints);
  }

  @Override
  public Continuation<FluentComponent> browseEager(final int limit, @Nullable final String continuationToken) {
    return doBrowseEager(limit, continuationToken, null, null, null);
  }

  @Override
  public Continuation<FluentComponent> browseWithAssets(final int limit, @Nullable final String continuationToken) {
    return new FluentContinuation<>(
        componentStore.browseComponentsWithAssets(facet.contentRepositoryId(), limit, continuationToken),
        this::with);
  }

  Continuation<FluentComponent> doBrowseEager(
      final int limit,
      @Nullable final String continuationToken,
      @Nullable final String kind,
      @Nullable final String filter,
      @Nullable final Map<String, Object> filterParams)
  {
    Set<Integer> repositoryIds = getRepositoryIds(null, facet, facet.repository());

    Continuation<ComponentData> componentAssetsData = componentStore
        .browseComponentsEager(repositoryIds, limit, continuationToken, kind, filter, filterParams);

    return new FluentContinuation<>(
        componentAssetsData,
        componentData -> {
          assert componentData != null;

          List<Asset> assets = componentData.getAssets();

          return facet.components().with(componentData, assets);
        });
  }

  Continuation<FluentComponent> doBrowse(
      final int limit,
      @Nullable final String continuationToken,
      @Nullable final String kind,
      @Nullable final String filter,
      @Nullable final Map<String, Object> filterParams,
      @Nullable final List<FluentQueryConstraint> constraints)
  {
    Set<Integer> repositoryIds = getRepositoryIds(constraints, facet, facet.repository());

    if (repositoryIds.size() > 1) {
      // with more than 1 repository, the kind/filter/filterParams all get ignored
      return new FluentContinuation<>(componentStore.browseComponents(repositoryIds, limit, continuationToken),
          this::with);
    }
    return new FluentContinuation<>(componentStore.browseComponents(repositoryIds.iterator().next(),
        limit, continuationToken, kind, filter, filterParams), this::with);
  }

  @Override
  public FluentQuery<FluentComponent> withGroupMemberContent() {
    return new FluentComponentQueryImpl(this, singletonList(new GroupRepositoryConstraint(BOTH)));
  }

  @Override
  public FluentQuery<FluentComponent> withOnlyGroupMemberContent() {
    return new FluentComponentQueryImpl(this, singletonList(new GroupRepositoryConstraint(MEMBERS)));
  }

  @Override
  public FluentQuery<FluentComponent> byKind(final String kind) {
    return new FluentComponentQueryImpl(this, kind);
  }

  @Override
  public FluentQuery<FluentComponent> byFilter(final String filter, final Map<String, Object> filterParams) {
    return new FluentComponentQueryImpl(this, filter, filterParams);
  }

  @Override
  public Continuation<FluentComponent> bySet(
      final ComponentSet componentSet,
      final int limit,
      final String continuationToken)
  {
    return new FluentContinuation<>(componentStore.browseComponentsBySet(facet.contentRepositoryId(),
        componentSet, limit, continuationToken), this::with);
  }

  @Override
  public Continuation<FluentComponent> selectComponents(
      final SqlGenerator<? extends SqlQueryParameters> generator,
      final SqlQueryParameters params)
  {
    return new FluentContinuation<>(componentStore.selectComponents(generator, params), this::with);
  }

  @Override
  public Continuation<FluentComponent> selectComponentsWithAssets(
      final SqlGenerator<? extends SqlQueryParameters> generator,
      final SqlQueryParameters params)
  {
    return new FluentContinuation<>(componentStore.selectComponentsWithAssets(generator, params), this::with);
  }

  @Override
  public Collection<String> namespaces() {
    return componentStore.browseNamespaces(facet.contentRepositoryId());
  }

  @Override
  public Collection<String> names(final String namespace) {
    return componentStore.browseNames(facet.contentRepositoryId(), namespace);
  }

  @Override
  public Continuation<ComponentSetData> sets(final int limit, final String continuationToken) {
    return componentStore.browseSets(facet.contentRepositoryId(), limit, continuationToken);
  }

  @Override
  public Collection<String> versions(final String namespace, final String name) {
    return componentStore.browseVersions(facet.contentRepositoryId(), namespace, name);
  }

  @Override
  public Collection<String> memberVersions(final String namespace, final String name) {
    if (!isGroupRepository(facet.repository())) {
      throw new IllegalArgumentException(facet.repository() + " is not a group repository");
    }

    List<FluentQueryConstraint> membersConstraint = singletonList(new GroupRepositoryConstraint(MEMBERS));
    Set<Integer> repositoryIds = getRepositoryIds(membersConstraint, facet, facet.repository());

    return componentStore.browseVersionsByRepoIds(namespace, name, repositoryIds);
  }

  @Override
  public Optional<FluentComponent> find(final EntityId externalId) {
    return componentStore.readComponent(toInternalId(externalId))
        .filter(this::containedInRepository)
        .map(component -> new FluentComponentImpl(facet, component));
  }

  /**
   * Returns {@code true} if this component is contained in this repository or any of its members.
   */
  private boolean containedInRepository(final Component component) {
    int expectedContentRepositoryId = contentRepositoryId(component);
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

  private boolean isNugetV2Proxy() {
    if (!facet.repository().getFormat().getValue().equals("nuget") ||
        !facet.repository().getType().getValue().equals("proxy")) {
      return false;
    }

    Configuration conf = facet.repository().getConfiguration();
    if (conf == null || conf.getAttributes() == null) {
      return false;
    }

    Map<String, Object> nugetProxy = conf.getAttributes().get("nugetProxy");
    if (nugetProxy == null) {
      return false;
    }

    String nugetVersion = (String) nugetProxy.get("nugetVersion");
    return "V2".equals(nugetVersion);
  }
}
