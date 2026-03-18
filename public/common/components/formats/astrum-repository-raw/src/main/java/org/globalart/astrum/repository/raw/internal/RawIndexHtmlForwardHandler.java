
package org.globalart.astrum.repository.raw.internal;

import javax.annotation.Nonnull;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.handlers.IndexHtmlForwardHandler;
import org.springframework.stereotype.Component;

/**
 * Handler which will forward current request to {@code {request.path}/index.html} or {@code {request.path}/index.htm}.
 * If there are no index.html and index.htm files then just proceed to avoid 404 error
 *
 * @since 3.29
 */
@Component
public class RawIndexHtmlForwardHandler
    extends IndexHtmlForwardHandler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Response response = forward(context, context.getRequest().getPath() + ".");
    if (HttpStatus.NOT_FOUND == response.getStatus().getCode()) {
      response = super.handle(context);
      if (HttpStatus.NOT_FOUND == response.getStatus().getCode()) {
        return HttpResponses.notFound("You can’t browse this way");
      }
    }

    return response;
  }
}
