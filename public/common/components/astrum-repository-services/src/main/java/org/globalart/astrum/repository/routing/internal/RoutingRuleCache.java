
package org.globalart.astrum.repository.routing.internal;

import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;
import org.globalart.astrum.repository.routing.RoutingRule;
import org.globalart.astrum.repository.routing.RoutingRuleInvalidatedEvent;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import com.google.common.cache.CacheBuilder;
import com.google.common.cache.CacheLoader;
import com.google.common.cache.LoadingCache;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * An in-memory cache of the RoutingRule assigned to a Repository. Uses events to know when to invalidate the
 * cache.
 *
 * @since 3.17
 */
@Component
public class RoutingRuleCache
    implements EventAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final LoadingCache<Repository, Optional<EntityId>> repositoryAssignedCache =
      CacheBuilder.newBuilder().build(new RepositoryMappingCacheLoader());

  private final LoadingCache<EntityId, Optional<RoutingRule>> routingRuleCache =
      CacheBuilder.newBuilder().build(new RoutingRuleCacheLoader());

  private final RoutingRuleStore routingRuleStore;

  @Autowired
  public RoutingRuleCache(final RoutingRuleStore routingRuleStore) {
    this.routingRuleStore = checkNotNull(routingRuleStore);
  }

  /**
   * Retrieves the routing rule assigned to a repository or null if one is not assigned.
   *
   * @param repository
   * @return
   */
  @Nullable
  public RoutingRule getRoutingRule(final Repository repository) {
    try {
      return repositoryAssignedCache.get(repository).map(this::getRoutingRule).orElse(null);
    }
    catch (ExecutionException e) {
      log.error("An error occurred retrieving the routing rule for repository: {}", repository.getName(), e);
      return null;
    }
  }

  /**
   * Retrieves the id of the routing rule assigned to a repository or null if one is not assigned.
   *
   * @param repository
   * @return
   */
  @Nullable
  public EntityId getRoutingRuleId(final Repository repository) {
    try {
      return repositoryAssignedCache.get(repository).orElse(null);
    }
    catch (ExecutionException e) {
      log.error("An error occurred retrieving the routing rule for repository: {}", repository.getName(), e);
      return null;
    }
  }

  private RoutingRule getRoutingRule(final EntityId id) {
    try {
      return routingRuleCache.get(id).orElse(null);
    }
    catch (ExecutionException e) {
      log.error("An error occurred retrieving the routing rule id {}", id, e);
      return null;
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final RepositoryDeletedEvent event) {
    repositoryAssignedCache.invalidate(event.getRepository());
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final RepositoryUpdatedEvent event) {
    repositoryAssignedCache.invalidate(event.getRepository());
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final RoutingRuleInvalidatedEvent event) {
    routingRuleCache.invalidate(new EntityUUID(UUID.fromString(event.getRoutingRuleId())));
  }

  private static class RepositoryMappingCacheLoader
      extends CacheLoader<Repository, Optional<EntityId>>
  {
    @Override
    public Optional<EntityId> load(final Repository repository) throws Exception {
      return Optional.ofNullable(repository.getConfiguration().getRoutingRuleId());
    }
  }

  private class RoutingRuleCacheLoader
      extends CacheLoader<EntityId, Optional<RoutingRule>>
  {
    @Override
    public Optional<RoutingRule> load(final EntityId key) throws Exception {
      return Optional.ofNullable(routingRuleStore.getById(key.getValue()));
    }
  }
}
