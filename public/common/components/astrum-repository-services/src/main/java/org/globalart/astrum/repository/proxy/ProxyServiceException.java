
package org.globalart.astrum.repository.proxy;

import javax.annotation.Nonnull;

import org.apache.http.HttpResponse;

/**
 * A format-neutral proxy service exception thrown in cases like proxy with misconfiguration or remote down.
 *
 * @since 3.0
 */
public class ProxyServiceException
    extends RuntimeException
{
  private final HttpResponse httpResponse;

  public ProxyServiceException(final HttpResponse httpResponse) {
    super(httpResponse.getStatusLine().toString());
    this.httpResponse = httpResponse;
  }

  public ProxyServiceException(final String message) {
    super(message);
    this.httpResponse = null;
  }

  /**
   * Returns the {@link HttpResponse} but with a <b>consumed entity</b>, to be able to inspect response status and
   * headers, if needed.
   */
  @Nonnull
  public HttpResponse getHttpResponse() {
    return httpResponse;
  }
}
