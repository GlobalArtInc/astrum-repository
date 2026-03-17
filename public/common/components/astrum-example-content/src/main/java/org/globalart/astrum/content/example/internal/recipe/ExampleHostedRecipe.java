
package org.globalart.astrum.content.example.internal.recipe;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.SuffixMatcher;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import static org.globalart.astrum.repository.view.matchers.logic.LogicMatchers.and;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Example hosted recipe.
 *
 * @since 3.24
 */
@Component
@Qualifier(ExampleHostedRecipe.NAME)
public class ExampleHostedRecipe
    extends ExampleRecipeSupport
{
  public static final String NAME = "example-hosted";

  @Autowired
  public ExampleHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(ExampleFormat.NAME) final Format format)
  {
    super(type, format);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
  }

  /**
   * Configure {@link ViewFacet}.
   */
  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    // Additional handlers, such as the lastDownloadHandler, are intentionally
    // not included on this route because this route forwards to the route below.
    // This route specifically handles GET / and forwards to /index.html.
    builder.route(new Route.Builder()
        .matcher(and(new ActionMatcher(HttpMethods.GET), new SuffixMatcher("/")))
        .handler(timingHandler)
        .handler(indexHtmlForwardHandler)
        .create());

    builder.route(new Route.Builder()
        .matcher(new TokenMatcher("{path:/.+}"))
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(contentHandler)
        .create());

    builder.defaultHandlers(HttpHandlers.badRequest());

    facet.configure(builder.create());

    return facet;
  }
}
