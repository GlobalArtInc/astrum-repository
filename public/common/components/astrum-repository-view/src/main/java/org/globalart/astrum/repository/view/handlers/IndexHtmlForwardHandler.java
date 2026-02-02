
package org.globalart.astrum.repository.view.handlers;

import javax.annotation.Nonnull;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Handler which will forward current request to {@code {request.path}/index.html} or {@code {request.path}/index.htm}.
 *
 * @since 3.0
 */
@Primary
@Component
public class IndexHtmlForwardHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String[] INDEX_FILES = {
      "index.html",
      "index.htm"
  };

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    String path = context.getRequest().getPath();

    // sanity, to ensure we don't corrupt the path
    if (!path.endsWith("/")) {
      path = path + "/";
    }

    for (String file : INDEX_FILES) {
      Response response = forward(context, path + file);
      // return response if it was successful or an error which was not not found
      if (response.getStatus().isSuccessful() || response.getStatus().getCode() != HttpStatus.NOT_FOUND) {
        return response;
      }
      // otherwise try next index file
    }

    // or there is no such file, give up not found
    return HttpResponses.notFound(context.getRequest().getPath());
  }

  protected Response forward(final Context context, final String path) throws Exception {
    log.trace("Forwarding request to path: {}", path);

    Request request = new Request.Builder()
        .copy(context.getRequest())
        .path(path)
        .build();

    return context.getRepository()
        .facet(ViewFacet.class)
        .dispatch(request);
  }
}
