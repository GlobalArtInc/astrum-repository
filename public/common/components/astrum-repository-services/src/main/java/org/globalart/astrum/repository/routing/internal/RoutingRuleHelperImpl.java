
package org.globalart.astrum.repository.routing.internal;

import java.util.List;
import java.util.Map;
import java.util.stream.StreamSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.routing.RoutingMode;
import org.globalart.astrum.repository.routing.RoutingRule;
import org.globalart.astrum.repository.routing.RoutingRuleHelper;
import org.globalart.astrum.repository.security.RepositoryAdminPermission;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;
import org.globalart.astrum.repository.types.ProxyType;

import org.apache.shiro.authz.Permission;
import org.springframework.context.annotation.Lazy;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.singletonList;
import static java.util.stream.Collectors.groupingBy;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.security.BreadActions.ADD;
import static org.globalart.astrum.security.BreadActions.READ;
import org.springframework.stereotype.Component;

/**
 * @since 3.16
 */
@Component
public class RoutingRuleHelperImpl
    implements RoutingRuleHelper
{
  private final RoutingRuleCache routingRuleCache;

  private final RepositoryManager repositoryManager;

  private final RepositoryPermissionChecker repositoryPermissionChecker;

  private volatile List<Permission> repositoryAddPermissions;

  @Autowired
  public RoutingRuleHelperImpl(
      final RoutingRuleCache routingRuleCache,
      @Lazy final RepositoryManager repositoryManager,
      @Lazy final RepositoryPermissionChecker repositoryPermissionChecker)
  {
    this.routingRuleCache = checkNotNull(routingRuleCache);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.repositoryPermissionChecker = checkNotNull(repositoryPermissionChecker);
  }

  @Override
  public boolean isAllowed(final Repository repository, final String path) {
    RoutingRule routingRule = routingRuleCache.getRoutingRule(repository);

    if (routingRule == null) {
      return true;
    }

    return isAllowed(routingRule, path);
  }

  @Override
  public boolean isAllowed(final RoutingRule routingRule, final String path) {
    return isAllowed(routingRule.mode(), routingRule.matchers(), path);
  }

  @Override
  public boolean isAllowed(final RoutingMode mode, final List<String> matchers, final String path) {
    boolean matches = matchers.stream().anyMatch(path::matches);
    return (!matches && mode == RoutingMode.BLOCK) || (matches && mode == RoutingMode.ALLOW);
  }

  @Override
  public Map<EntityId, List<Repository>> calculateAssignedRepositories() {
    return StreamSupport.stream(repositoryManager.browse().spliterator(), false)
        .filter(repository -> routingRuleCache.getRoutingRuleId(repository) != null)
        .collect(groupingBy(routingRuleCache::getRoutingRuleId, toList()));
  }

  @Override
  public boolean hasRoutingRule(final Repository repository) {
    RoutingRule routingRule = routingRuleCache.getRoutingRule(repository);

    return routingRule != null;
  }

  @Override
  public void ensureUserHasPermissionToRead() {
    List<Permission> permissions = getRepositoryAddPermissions();
    if (!permissions.isEmpty()) { // avoid log-spam when we start Astrum with no recipes
      repositoryPermissionChecker.ensureUserHasAnyPermissionOrAdminAccess(
          permissions,
          READ,
          repositoryManager.browse());
    }
  }

  private List<Permission> getRepositoryAddPermissions() {
    if (null == repositoryAddPermissions) {
      repositoryAddPermissions = repositoryManager.getAllSupportedRecipes()
          .stream()
          .filter(r -> r.getType().getValue().equals(ProxyType.NAME))
          .map(r -> new RepositoryAdminPermission(r.getFormat().getValue(), "*", singletonList(ADD)))
          .collect(toList());
    }
    return repositoryAddPermissions;
  }
}
