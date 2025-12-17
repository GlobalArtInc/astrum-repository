
package org.globalart.astrum.security.authc;

import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.HostAuthenticationToken;

/**
 * {@link AuthenticationToken} that contains credentials from a known API-Key.
 */
public class AstrumApiKeyAuthenticationToken
    implements HostAuthenticationToken
{
  private Object principal;

  private final char[] credentials;

  private final String host;

  public AstrumApiKeyAuthenticationToken(final Object principal, final char[] credentials, final String host) {
    this.principal = principal;
    this.credentials = credentials;
    this.host = host;
  }

  public Object getPrincipal() {
    return principal;
  }

  public Object getCredentials() {
    return credentials;
  }

  public String getHost() {
    return host;
  }

  /**
   * Assigns a new account identity to the current authentication token.
   */
  public void setPrincipal(final Object principal) {
    this.principal = principal;
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
