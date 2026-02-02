
package org.globalart.astrum.repository.view.handlers;

import java.net.URL;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import com.google.common.collect.ImmutableList;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Renders browse not supported page for requests to "/" or "/index.html" for
 * repositories that do not generate this information.
 *
 * @since 3.0
 */
@Component
public class BrowseUnsupportedHandler
    implements Handler, org.globalart.astrum.repository.BrowseUnsupportedHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String TEMPLATE_RESOURCE = "browseUnsupportedHtml.vm";

  private final TemplateHelper templateHelper;

  private final URL template;

  private final Route route;

  @Autowired
  public BrowseUnsupportedHandler(final TemplateHelper templateHelper, final SecurityHandler securityHandler) {
    this.templateHelper = checkNotNull(templateHelper);
    checkNotNull(securityHandler);
    this.template = getClass().getResource(TEMPLATE_RESOURCE);
    checkNotNull(template);
    this.route = new Route(MATCHER, ImmutableList.of(securityHandler, this));
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    TemplateParameters params = templateHelper.parameters();
    params.set("repository", context.getRepository());

    String html = templateHelper.render(template, params);
    return HttpResponses.ok(new StringPayload(html, "text/html"));
  }

  @Override
  @Nonnull
  public Route getRoute() {
    return this.route;
  }

  //
  // Matcher
  //

  /**
   * Matches GET request with path ending with one of (ignoring case):
   *
   * "/"
   * "/index.html"
   * "/index.htm"
   */
  private static class MatcherImpl
      implements Matcher
  {
    protected final Logger log = LoggerFactory.getLogger(getClass());

    @Override
    public boolean matches(final Context context) {
      checkNotNull(context);
      String action = context.getRequest().getAction();
      String path = context.getRequest().getPath();
      log.debug("Matching: {} {}", action, path);
      if (HttpMethods.GET.equals(action) || HttpMethods.HEAD.equals(action)) {
        path = Strings2.lower(path);

        return path.endsWith("/") || path.endsWith("/index.html") || path.endsWith("/index.htm");
      }
      return false;
    }
  }

  private static final Matcher MATCHER = new MatcherImpl();
}
