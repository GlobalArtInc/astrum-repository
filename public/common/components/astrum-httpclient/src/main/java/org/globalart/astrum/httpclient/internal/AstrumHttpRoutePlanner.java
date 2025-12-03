
package org.globalart.astrum.httpclient.internal;

import java.util.Map;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.apache.http.HttpException;
import org.apache.http.HttpHost;
import org.apache.http.HttpRequest;
import org.apache.http.conn.routing.HttpRoutePlanner;
import org.apache.http.impl.conn.DefaultRoutePlanner;
import org.apache.http.impl.conn.DefaultSchemePortResolver;
import org.apache.http.protocol.HttpContext;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An {@link HttpRoutePlanner} that uses different proxies / url scheme (http/https) and bypasses proxy for specific
 * hosts (non proxy hosts).
 *
 * @since 2.6
 */
public class AstrumHttpRoutePlanner
    extends DefaultRoutePlanner
{
  /**
   * Single regexp pattern covering non-proxy-hosts, never {@code null}
   */
  private final Pattern nonProxyHostPattern;

  /**
   * Mapping between protocol scheme and proxy to be used
   */
  private final Map<String, HttpHost> proxies;

  /**
   * @since 2.5
   */
  public AstrumHttpRoutePlanner(
      final Map<String, HttpHost> proxies,
      @Nullable final Pattern nonProxyHostsPattern)
  {
    super(DefaultSchemePortResolver.INSTANCE);
    this.proxies = checkNotNull(proxies);
    this.nonProxyHostPattern = nonProxyHostsPattern;
  }

  @Override
  protected HttpHost determineProxy(
      final HttpHost host,
      final HttpRequest request,
      final HttpContext context) throws HttpException
  {
    if (noProxyFor(host.getHostName())) {
      return null;
    }
    return proxies.get(host.getSchemeName());
  }

  // TODO: Should we guard this with a cache to avoid rematching regex constantly?

  /**
   * Determine if proxy should be configured for given host or not.
   *
   * @return true if no proxy should be configured.
   */
  private boolean noProxyFor(final String hostName) {
    return nonProxyHostPattern != null && nonProxyHostPattern.matcher(hostName).matches();
  }
}
