
package org.globalart.astrum.bootstrap.jetty;

import org.eclipse.jetty.http.HttpScheme;

public class UnsupportedHttpSchemeException
    extends IllegalArgumentException
{
  private final HttpScheme httpScheme;

  public UnsupportedHttpSchemeException(final HttpScheme httpScheme) {
    super("Unsupported HTTP Scheme: " + httpScheme);
    this.httpScheme = httpScheme;
  }

  public HttpScheme getHttpScheme() {
    return httpScheme;
  }
}
