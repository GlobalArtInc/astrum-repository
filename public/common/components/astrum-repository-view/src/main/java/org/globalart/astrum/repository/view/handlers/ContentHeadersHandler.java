
package org.globalart.astrum.repository.view.handlers;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.ETagHeaderUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

import com.google.common.net.HttpHeaders;
import org.joda.time.DateTime;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.date.DateTimeUtils.formatDateTime;

/**
 * A format-neutral content handler for decorating response using {@link Content#getAttributes()} provided attributes.
 *
 * @since 3.0
 */
@Component
public class ContentHeadersHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    final Response response = context.proceed();
    Payload payload = response.getPayload();

    if (response.getStatus().isSuccessful() && payload instanceof Content content) {
      final DateTime lastModified = content.getAttributes().get(Content.CONTENT_LAST_MODIFIED, DateTime.class);
      if (lastModified != null) {
        response.getHeaders().replace(HttpHeaders.LAST_MODIFIED, formatDateTime(lastModified));
      }
      final String etag = content.getAttributes().get(Content.CONTENT_ETAG, String.class);
      if (etag != null) {
        response.getHeaders().replace(HttpHeaders.ETAG, ETagHeaderUtils.quote(etag));
      }
      final String vary = content.getAttributes().get(Content.CONTENT_VARY, String.class);
      if (vary != null) {
        response.getHeaders().replace(HttpHeaders.VARY, vary);
      }
    }
    return response;
  }
}
