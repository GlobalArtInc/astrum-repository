
package org.globalart.astrum.repository.p2.internal.proxy;

import java.util.Objects;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;
import org.globalart.astrum.repository.p2.P2ContentFacet;
import org.globalart.astrum.repository.p2.internal.P2Format;
import org.globalart.astrum.repository.proxy.ProxyFacet;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Qualifier(P2Format.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Facet.Exposed
public class P2ProxyCacheInvalidatorFacet
    extends FacetSupport
{
  @Subscribe
  @AllowConcurrentEvents
  protected void on(final RepositoryUpdatedEvent event) {
    Repository repository = event.getRepository();
    if (!repository.getName().equals(getRepository().getName())) {
      return;
    }
    if (!Objects.equals(getRemoteUrl(repository.getConfiguration()), getRemoteUrl(event.getOldConfiguration()))) {
      log.info("Remote URL of p2 proxy repository {} changed, removing cached content", repository.getName());
      deleteAssets();
    }
  }

  public void deleteAssets() {
    Repository repository = getRepository();
    repository.facet(P2ContentFacet.class).deleteAll();
    repository.optionalFacet(ProxyFacet.class).ifPresent(ProxyFacet::invalidateProxyCaches);
  }

  private static Object getRemoteUrl(final Configuration configuration) {
    return Optional.ofNullable(configuration.getAttributes().get("proxy"))
        .map(proxy -> proxy.get("remoteUrl"))
        .orElse(null);
  }
}
