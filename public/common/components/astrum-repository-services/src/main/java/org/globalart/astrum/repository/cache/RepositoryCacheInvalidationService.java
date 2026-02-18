
package org.globalart.astrum.repository.cache;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.distributed.event.service.api.common.RepositoryCacheInvalidationEvent;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.proxy.ProxyFacet;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.ProxyType;

import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Service class for consolidating repeated cache-related logic not exclusive to individual facets and components.
 *
 * @since 3.41
 */
@Component
public class RepositoryCacheInvalidationService
    implements EventAware
{
  private final RepositoryManager repositoryManager;

  private final EventManager eventManager;

  @Autowired
  public RepositoryCacheInvalidationService(
      final RepositoryManager repositoryManager,
      final EventManager eventManager)
  {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.eventManager = checkNotNull(eventManager);
  }

  public void processCachesInvalidation(final Repository repository) {
    invalidateCaches(repository, true);
  }

  /**
   * Invalidates the group or proxy caches of the specified repository based on type and fires an DES event if cache
   * invalidation was triggered by current node.
   * This is a no-op for hosted repositories.
   */
  private void invalidateCaches(final Repository repository, final boolean isTriggeredByCurrentNode) {
    checkNotNull(repository);
    if (GroupType.NAME.equals(repository.getType().getValue())) {
      invalidateGroupCaches(repository);
    }
    else if (ProxyType.NAME.equals(repository.getType().getValue())) {
      invalidateProxyAndNegativeCaches(repository);
    }
    if (isTriggeredByCurrentNode) {
      postEvent(repository);
    }
  }

  /**
   * Invalidates the group caches for given repository.
   */
  private static void invalidateGroupCaches(final Repository repository) {
    checkNotNull(repository);
    checkArgument(GroupType.NAME.equals(repository.getType().getValue()));
    GroupFacet groupFacet = repository.facet(GroupFacet.class);
    groupFacet.invalidateGroupCaches();
  }

  /**
   * Invalidates the proxy and negative caches for given repository.
   */
  private static void invalidateProxyAndNegativeCaches(final Repository repository) {
    checkNotNull(repository);
    checkArgument(ProxyType.NAME.equals(repository.getType().getValue()));
    ProxyFacet proxyFacet = repository.facet(ProxyFacet.class);
    proxyFacet.invalidateProxyCaches();
    NegativeCacheFacet negativeCacheFacet = repository.facet(NegativeCacheFacet.class);
    negativeCacheFacet.invalidate();
  }

  private void postEvent(final Repository repository) {
    if (!EventHelper.isReplicating()) {
      eventManager.post(new RepositoryCacheInvalidationEvent(repository.getName()));
    }
  }

  @Subscribe
  public void on(final RepositoryCacheInvalidationEvent event) {
    invalidateCaches(repositoryManager.get(event.getRepositoryName()), false);
  }
}
