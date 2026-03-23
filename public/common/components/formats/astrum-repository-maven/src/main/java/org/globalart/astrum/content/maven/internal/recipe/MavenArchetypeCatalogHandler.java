
package org.globalart.astrum.content.maven.internal.recipe;

import javax.annotation.Nonnull;
import org.globalart.astrum.content.maven.MavenArchetypeCatalogFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import org.springframework.stereotype.Component;

/**
 * Fetches or rebuilds the maven archetype catalog for a given repository.
 *
 * @since 3.25
 */
@Component
public class MavenArchetypeCatalogHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Nonnull
  @Override
  public Response handle(
      @Nonnull final Context context) throws Exception
  {
    String method = context.getRequest().getAction();
    switch (method) {
      case GET:
      case HEAD:
        return fetchOrGenerateArchetypeCatalog(context);
      default:
        return HttpResponses.methodNotAllowed(context.getRequest().getAction(), GET, HEAD);
    }
  }

  private Response fetchOrGenerateArchetypeCatalog(final Context context) throws Exception {
    Response response = context.proceed();
    if (!response.getStatus().isSuccessful()) {
      return generateArchetypeCatalog(context);
    }
    else {
      return response;
    }
  }

  private Response generateArchetypeCatalog(final Context context) throws Exception {
    Repository repository = context.getRepository();
    MavenArchetypeCatalogFacet archetypeCatalogFacet = repository.facet(MavenArchetypeCatalogFacet.class);
    archetypeCatalogFacet.rebuildArchetypeCatalog();
    return context.proceed();
  }
}
