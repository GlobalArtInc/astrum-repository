
package org.globalart.astrum.repository.http;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

/**
 * Helper to create common HTTP {@link Handler} instances.
 *
 * @since 3.0
 */
public class HttpHandlers
{
  private HttpHandlers() {
    // empty
  }

  public static Handler notFound() {
    return new Handler()
    {
      @Nonnull
      @Override
      public Response handle(@Nonnull final Context context) throws Exception {
        return HttpResponses.notFound();
      }
    };
  }

  public static Handler badRequest() {
    return new Handler()
    {
      @Nonnull
      @Override
      public Response handle(@Nonnull final Context context) throws Exception {
        return HttpResponses.badRequest();
      }
    };
  }
}
