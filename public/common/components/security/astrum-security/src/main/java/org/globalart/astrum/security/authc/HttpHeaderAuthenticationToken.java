
package org.globalart.astrum.security.authc;

import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.HostAuthenticationToken;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link AuthenticationToken} with a principal extracted from an HTTP header.
 *
 * The principal is equal to HTTP header value.
 *
 * @since 2.7
 */
public class HttpHeaderAuthenticationToken
    implements HostAuthenticationToken
{
  private final String headerName;

  private final String headerValue;

  private final String host;

  public HttpHeaderAuthenticationToken(final String headerName, final String headerValue, final String host) {
    this.headerName = checkNotNull(headerName);
    this.headerValue = checkNotNull(headerValue);
    this.host = host;
  }

  @Override
  public String getPrincipal() {
    return headerValue;
  }

  @Override
  public Object getCredentials() {
    return null;
  }

  @Override
  public String getHost() {
    return host;
  }

  public String getHeaderName() {
    return headerName;
  }

  public String getHeaderValue() {
    return headerValue;
  }

  @Override
  public String toString() {
    final StringBuilder buf = new StringBuilder(getClass().getName());
    buf.append(" - ").append(getPrincipal());
    if (host != null) {
      buf.append(" (").append(host).append(")");
    }
    return buf.toString();
  }
}
