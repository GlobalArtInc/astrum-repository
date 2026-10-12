package org.globalart.astrum.repository.rubygems.internal.hosted;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.RubygemsPaths;
import org.globalart.astrum.repository.rubygems.internal.RubygemsRecipeSupport;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(RubygemsHostedRecipe.NAME)
public class RubygemsHostedRecipe
    extends RubygemsRecipeSupport
{
  public static final String NAME = "rubygems-hosted";

  private final Provider<RubygemsHostedFacet> hostedFacet;

  private final RubygemsHostedHandlers hostedHandlers;

  @Autowired
  public RubygemsHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(RubygemsFormat.NAME) final Format format,
      final Provider<RubygemsHostedFacet> hostedFacet,
      final RubygemsHostedHandlers hostedHandlers)
  {
    super(type, format);
    this.hostedFacet = checkNotNull(hostedFacet);
    this.hostedHandlers = checkNotNull(hostedHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(hostedFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addCommonRoutes(builder);
    builder.route(apiRoute(RubygemsPaths.push(), hostedHandlers.push));
    builder.route(apiRoute(RubygemsPaths.yank(), hostedHandlers.yank));
    builder.route(indexRoute(RubygemsPaths.versions(), hostedHandlers.versions));
    builder.route(indexRoute(RubygemsPaths.names(), hostedHandlers.names));
    builder.route(indexRoute(RubygemsPaths.info(), hostedHandlers.info));
    builder.route(contentRoute(RubygemsPaths.legacySpecs(), hostedHandlers.legacySpecs));
    builder.route(contentRoute(RubygemsPaths.quickSpec(), hostedHandlers.quickSpec));
    builder.route(contentRoute(RubygemsPaths.gem(), lastDownloadedHandler, hostedHandlers.gem));
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }
}
