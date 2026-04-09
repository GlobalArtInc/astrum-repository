package org.globalart.astrum.repository.helm.internal.content.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.helm.internal.AssetKind;
import org.globalart.astrum.repository.helm.internal.HelmFormat;
import org.globalart.astrum.repository.helm.internal.content.createindex.CreateIndexFacetImpl;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(HelmHostedRecipe.NAME)
public class HelmHostedRecipe
    extends HelmRecipeSupport
{
  public static final String NAME = "helm-hosted";

  private final HostedHandlers hostedHandlers;

  private final Provider<CreateIndexFacetImpl> createIndexFacet;

  private final Provider<HelmHostedFacetImpl> hostedFacet;

  @Autowired
  public HelmHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(HelmFormat.NAME) final Format format,
      final HostedHandlers hostedHandlers,
      final Provider<CreateIndexFacetImpl> createIndexFacet,
      final Provider<HelmHostedFacetImpl> hostedFacet)
  {
    super(type, format);
    this.hostedHandlers = checkNotNull(hostedHandlers);
    this.createIndexFacet = checkNotNull(createIndexFacet);
    this.hostedFacet = checkNotNull(hostedFacet);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(hostedFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(createIndexFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addBrowseUnsupportedRoute(builder);
    for (Matcher matcher : new Matcher[]{indexMatcher(), packageMatcher()}) {
      builder.route(route(matcher, false, hostedHandlers.get));
    }
    builder.route(route(methodMatcher("tgz", AssetKind.HELM_PACKAGE, PUT), true, hostedHandlers.upload));
    builder.route(route(methodMatcher("tgz.prov", AssetKind.HELM_PROVENANCE, PUT), true, hostedHandlers.upload));
    builder.route(route(pushMatcher(), true, hostedHandlers.push));
    builder.route(route(methodMatcher("tgz", AssetKind.HELM_PACKAGE, DELETE), true, hostedHandlers.delete));
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route route(final Matcher matcher, final boolean conditional, final Handler last) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
    if (conditional) {
      route.handler(conditionalRequestHandler);
    }
    return route.handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(last)
        .create();
  }
}
