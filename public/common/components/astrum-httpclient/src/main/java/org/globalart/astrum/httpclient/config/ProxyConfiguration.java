
package org.globalart.astrum.httpclient.config;

import java.util.Arrays;

import javax.annotation.Nullable;
import jakarta.validation.Valid;

/**
 * Proxy configuration.
 *
 * @since 3.0
 */
public class ProxyConfiguration
    implements Cloneable
{
  @Valid
  @Nullable
  private ProxyServerConfiguration http;

  @Valid
  @Nullable
  private ProxyServerConfiguration https;

  @NonProxyHosts
  @Nullable
  private String[] nonProxyHosts;

  @Nullable
  public ProxyServerConfiguration getHttp() {
    return http;
  }

  public void setHttp(@Nullable final ProxyServerConfiguration http) {
    this.http = http;
  }

  @Nullable
  public ProxyServerConfiguration getHttps() {
    return https;
  }

  public void setHttps(@Nullable final ProxyServerConfiguration https) {
    this.https = https;
  }

  @Nullable
  public String[] getNonProxyHosts() {
    return nonProxyHosts;
  }

  public void setNonProxyHosts(@Nullable final String[] nonProxyHosts) {
    this.nonProxyHosts = nonProxyHosts;
  }

  public ProxyConfiguration copy() {
    try {
      ProxyConfiguration copy = (ProxyConfiguration) clone();
      if (http != null) {
        copy.http = http.copy();
      }
      if (https != null) {
        copy.https = https.copy();
      }
      if (nonProxyHosts != null) {
        copy.nonProxyHosts = nonProxyHosts.clone();
      }
      return copy;
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "http=" + http +
        ", https=" + https +
        ", nonProxyHosts=" + Arrays.toString(nonProxyHosts) +
        '}';
  }
}
