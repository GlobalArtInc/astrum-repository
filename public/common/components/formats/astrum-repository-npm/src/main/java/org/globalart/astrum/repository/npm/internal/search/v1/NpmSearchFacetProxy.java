
package org.globalart.astrum.repository.npm.internal.search.v1;

import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.io.IOException;


import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.npm.internal.NpmProxyFacet.ProxyTarget;
import org.globalart.astrum.repository.proxy.ProxyFacet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Request;

import static org.globalart.astrum.repository.http.HttpMethods.GET;

/**
 * Implementation of {@code NpmSearchFacet} for proxy repositories.
 *
 * @since 3.7
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NpmSearchFacetProxy
    extends FacetSupport
    implements NpmSearchFacet
{
  private static final String REPOSITORY_SEARCH_ASSET = "-/v1/search";

  @Override
  public Content searchV1(final Parameters parameters) throws IOException {
    try {
      final Request getRequest = new Request.Builder()
          .action(GET)
          .path("/" + REPOSITORY_SEARCH_ASSET)
          .parameters(parameters)
          .build();
      Context context = new Context(getRepository(), getRequest);
      context.getAttributes().set(ProxyTarget.class, ProxyTarget.SEARCH_V1_RESULTS);
      Content searchResults = getRepository().facet(ProxyFacet.class).get(context);
      if (searchResults == null) {
        throw new IOException("Could not retrieve registry search");
      }
      return searchResults;
    }
    catch (Exception e) {
      throw new IOException(e);
    }
  }
}
