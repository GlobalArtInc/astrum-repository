
package org.globalart.astrum.internal.httpclient;

import javax.annotation.Nullable;
import jakarta.validation.Valid;

import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.ConnectionConfiguration;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;
import org.globalart.astrum.httpclient.config.ProxyConfiguration;

import org.apache.http.client.AuthenticationStrategy;
import org.apache.http.client.RedirectStrategy;

/**
 * {@link HttpClientConfiguration} data.
 *
 * @since 3.21
 */
public class HttpClientConfigurationData
    implements HttpClientConfiguration, Cloneable
{
  @Valid
  @Nullable
  private ConnectionConfiguration connection;

  @Valid
  @Nullable
  private ProxyConfiguration proxy;

  @Valid
  @Nullable
  private RedirectStrategy redirectStrategy;

  @Valid
  @Nullable
  private AuthenticationStrategy authenticationStrategy;

  @Valid
  @Nullable
  private Boolean disableContentCompression;

  /**
   * @see AuthenticationConfigurationDeserializer
   */
  @Valid
  @Nullable
  private AuthenticationConfiguration authentication;

  @Valid
  @Nullable
  private Boolean shouldNormalizeUri;

  @Override
  @Nullable
  public ConnectionConfiguration getConnection() {
    return connection;
  }

  @Override
  public void setConnection(@Nullable final ConnectionConfiguration connection) {
    this.connection = connection;
  }

  @Override
  @Nullable
  public ProxyConfiguration getProxy() {
    return proxy;
  }

  @Override
  public void setProxy(@Nullable final ProxyConfiguration proxy) {
    this.proxy = proxy;
  }

  @Override
  @Nullable
  public AuthenticationConfiguration getAuthentication() {
    return authentication;
  }

  @Override
  public void setAuthentication(@Nullable final AuthenticationConfiguration authentication) {
    this.authentication = authentication;
  }

  @Override
  @Nullable
  public RedirectStrategy getRedirectStrategy() {
    return redirectStrategy;
  }

  @Override
  public void setRedirectStrategy(@Nullable final RedirectStrategy redirectStrategy) {
    this.redirectStrategy = redirectStrategy;
  }

  @Nullable
  @Override
  public AuthenticationStrategy getAuthenticationStrategy() {
    return authenticationStrategy;
  }

  @Override
  public void setAuthenticationStrategy(
      @Nullable final AuthenticationStrategy authenticationStrategy)
  {
    this.authenticationStrategy = authenticationStrategy;
  }

  @Override
  public Boolean getNormalizeUri() {
    return shouldNormalizeUri;
  }

  @Override
  public void setNormalizeUri(final Boolean normalizeUri) {
    this.shouldNormalizeUri = normalizeUri;
  }

  @Override
  public Boolean getDisableContentCompression() {
    return disableContentCompression;
  }

  @Override
  public void setDisableContentCompression(final Boolean disableContentCompression) {
    this.disableContentCompression = disableContentCompression;
  }

  @Override
  public HttpClientConfigurationData copy() {
    try {
      HttpClientConfigurationData copy = (HttpClientConfigurationData) clone();
      if (connection != null) {
        copy.connection = connection.copy();
      }
      if (proxy != null) {
        copy.proxy = proxy.copy();
      }
      if (authentication != null) {
        copy.authentication = authentication.copy();
      }
      if (redirectStrategy != null) {
        // no real cloning/copying needed, as we are allowed to use a singleton instance
        copy.redirectStrategy = redirectStrategy;
      }
      if (authenticationStrategy != null) {
        // no real cloning/copying needed, as we are allowed to use a singleton instance
        copy.authenticationStrategy = authenticationStrategy;
      }
      if (disableContentCompression != null) {
        copy.disableContentCompression = disableContentCompression;
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
        "connection=" + connection +
        ", proxy=" + proxy +
        ", authentication=" + authentication +
        '}';
  }
}
