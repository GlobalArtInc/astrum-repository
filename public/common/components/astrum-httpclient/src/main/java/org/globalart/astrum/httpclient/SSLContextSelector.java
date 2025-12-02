
package org.globalart.astrum.httpclient;

import javax.net.ssl.SSLContext;

import org.apache.http.protocol.HttpContext;

/**
 * Selects SSLContext to be used for given HTTP context.
 *
 * @since 2.8
 */
public interface SSLContextSelector
{
  // TODO: If this is only ever used as context attribute name, consider better naming strategy.

  /**
   * {@link HttpContext} attribute to be used to enable usage of Astrum Repository Truststore.
   *
   * @since 3.0
   */
  String USE_TRUST_STORE = "astrum.httpclient.ssl.trustStore";

  /**
   * Returns the desired {@link SSLContext} to be used or {@code null} if no selection possible (or available).
   *
   * In this case, HTTP client will use the "default" SSL context.
   */
  SSLContext select(HttpContext context);
}
