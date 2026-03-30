
package org.globalart.astrum.repository.npm.internal;

import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.util.Map;
import java.util.Objects;
import java.util.Optional;


import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

/**
 * Handles invalidating NPM proxy cache when the URL for the repository changes.
 *
 * @since 3.21
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Facet.Exposed
public class NpmProxyCacheInvalidatorFacetImpl
    extends FacetSupport
{
  @Subscribe
  @AllowConcurrentEvents
  protected void on(final RepositoryUpdatedEvent event) {
    final Repository repository = event.getRepository();

    repository.optionalFacet(NpmProxyFacet.class).ifPresent(npm -> {
      if (!Objects.equals(getRemoteUrl(repository.getConfiguration()), getRemoteUrl(event.getOldConfiguration()))) {
        npm.invalidateProxyCaches();
      }
    });
  }

  private static Object getRemoteUrl(final Configuration configuration) {
    return Optional.ofNullable(configuration.getAttributes().get("proxy")).map(proxy -> {
      if (proxy instanceof Map) {
        return proxy.get("remoteUrl");
      }
      return null;
    }).orElse(null);
  }
}
