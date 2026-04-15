
package org.globalart.astrum.repository.golang.internal.content.hosted;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.golang.GolangFormat;
import org.globalart.astrum.repository.golang.internal.content.GoRecipeSupport;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(GoHostedRecipe.NAME)
public class GoHostedRecipe
    extends GoRecipeSupport
{
  public static final String NAME = "go-hosted";

  private final GoHostedHandlers hostedHandlers;

  @Autowired
  public GoHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(GolangFormat.NAME) final Format format,
      final GoHostedHandlers hostedHandlers)
  {
    super(type, format);
    this.hostedHandlers = checkNotNull(hostedHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addBrowseUnsupportedRoute(builder);
    for (Matcher matcher : new Matcher[]{infoMatcher(), listMatcher()}) {
      builder.route(route(matcher, false, hostedHandlers.get));
    }
    for (Matcher matcher : new Matcher[]{packageMatcher(), moduleMatcher()}) {
      builder.route(route(matcher, false, lastDownloadedHandler, hostedHandlers.get));
    }
    builder.route(route(uploadMatcher(), true, hostedHandlers.upload));
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route route(final Matcher matcher, final boolean conditional, final Handler... last) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
    if (conditional) {
      route.handler(conditionalRequestHandler);
    }
    route.handler(partialFetchHandler).handler(contentHeadersHandler);
    for (Handler handler : last) {
      route.handler(handler);
    }
    return route.create();
  }
}
