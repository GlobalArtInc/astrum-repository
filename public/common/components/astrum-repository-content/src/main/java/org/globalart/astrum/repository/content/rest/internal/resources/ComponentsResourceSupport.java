
package org.globalart.astrum.repository.content.rest.internal.resources;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.function.Predicate;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.rest.api.RepositoryManagerRESTAdapter;
import org.globalart.astrum.repository.selector.ContentAuthHelper;
import org.globalart.astrum.repository.types.GroupType;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.repository.content.rest.internal.resources.AssetsResourceSupport.toInternalToken;
import static org.globalart.astrum.repository.content.rest.internal.resources.AssetsResourceSupport.trim;

/**
 * Support class for {@link ComponentsResource} which fetches and returns only components that the user is permitted
 * to view according to {@link ContentAuthHelper#checkPathPermissions(String, String, String...)}
 *
 * @since 3.27
 */
abstract class ComponentsResourceSupport
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  /**
   * Limit the number of components returned per page. This value is aligned with AssetsResourceSupport.PAGE_SIZE_LIMIT
   * and also matches the number of component identifiers that can be passed as input to the Firewall component
   * evaluation API.
   */
  protected static final int PAGE_SIZE_LIMIT = 100;

  private final ContentAuthHelper contentAuthHelper;

  private final RepositoryManagerRESTAdapter repositoryManagerRESTAdapter;

  ComponentsResourceSupport(
      final ContentAuthHelper contentAuthHelper,
      final RepositoryManagerRESTAdapter repositoryManagerRESTAdapter)
  {
    this.contentAuthHelper = checkNotNull(contentAuthHelper);
    this.repositoryManagerRESTAdapter = checkNotNull(repositoryManagerRESTAdapter);
  }

  @VisibleForTesting
  List<FluentComponent> browse(final Repository browsedRepository, final String continuationToken) {
    List<FluentComponent> permittedComponents = new ArrayList<>();
    String internalToken = toInternalToken(continuationToken);
    Continuation<FluentComponent> componentContinuation = getComponents(browsedRepository, internalToken);

    while (permittedComponents.size() < PAGE_SIZE_LIMIT && !componentContinuation.isEmpty()) {
      permittedComponents.addAll(removeComponentsNotPermitted(browsedRepository, componentContinuation));
      componentContinuation = getComponents(browsedRepository, componentContinuation.nextContinuationToken());
    }
    return trim(permittedComponents, PAGE_SIZE_LIMIT);
  }

  private Continuation<FluentComponent> getComponents(final Repository repository, final String continuationToken) {
    if (GroupType.NAME.equals(repository.getType().getValue())) {
      return repository.facet(ContentFacet.class)
          .components()
          .withOnlyGroupMemberContent()
          .browse(PAGE_SIZE_LIMIT, continuationToken);
    }
    return repository.facet(ContentFacet.class).components().browse(PAGE_SIZE_LIMIT, continuationToken);
  }

  private List<FluentComponent> removeComponentsNotPermitted(
      final Repository repository,
      final Continuation<FluentComponent> assets)
  {
    return assets.stream()
        .filter(componentPermitted(repository.getFormat().getValue(), repository.getName()))
        .collect(toList());
  }

  Predicate<FluentComponent> componentPermitted(final String format, final String repositoryName) {
    return component -> contentAuthHelper.checkPathPermissions(component.name(), format, repositoryName);
  }

  Predicate<FluentAsset> assetPermitted(final Repository repository) {
    String repositoryName = repository.getName();
    Set<String> repoNames = new HashSet<>(repositoryManagerRESTAdapter.findContainingGroups(repositoryName));
    repoNames.add(repositoryName);
    return asset -> contentAuthHelper.checkPathPermissions(asset.path(), repository.getFormat().getValue(),
        repoNames.toArray(new String[0]));
  }
}
