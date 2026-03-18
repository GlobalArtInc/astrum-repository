
package org.globalart.astrum.repository.raw;

import javax.annotation.Nonnull;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import static org.globalart.astrum.repository.http.HttpMethods.GET;
import org.springframework.stereotype.Component;

/**
 * Handler to set Content-Disposition HTTP header
 *
 * @since 3.25
 */
@Component
public class ContentDispositionHandler
    implements Handler
{
  public static final String CONTENT_DISPOSITION_CONFIG_KEY = "contentDisposition";

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Response response = context.proceed();
    String action = context.getRequest().getAction();
    if (GET.equals(action)) {
      String contentDisposition = context.getRepository()
          .getConfiguration()
          .attributes("raw")
          .get(CONTENT_DISPOSITION_CONFIG_KEY, String.class, ContentDisposition.INLINE.name());
      response.getHeaders().replace("Content-Disposition", ContentDisposition.valueOf(contentDisposition).getValue());
    }
    return response;
  }
}
