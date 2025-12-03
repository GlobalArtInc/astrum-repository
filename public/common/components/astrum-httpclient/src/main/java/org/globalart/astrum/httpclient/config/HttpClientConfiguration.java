
package org.globalart.astrum.httpclient.config;

import javax.annotation.Nullable;

import org.apache.http.client.AuthenticationStrategy;
import org.apache.http.client.RedirectStrategy;

/**
 * HTTP-client configuration.
 *
 * @since 3.0
 */
public interface HttpClientConfiguration
{
  @Nullable
  ConnectionConfiguration getConnection();

  void setConnection(@Nullable final ConnectionConfiguration connection);

  @Nullable
  ProxyConfiguration getProxy();

  void setProxy(@Nullable final ProxyConfiguration proxy);

  @Nullable
  AuthenticationConfiguration getAuthentication();

  void setAuthentication(@Nullable final AuthenticationConfiguration authentication);

  @Nullable
  RedirectStrategy getRedirectStrategy();

  void setRedirectStrategy(@Nullable final RedirectStrategy redirectStrategy);

  @Nullable
  AuthenticationStrategy getAuthenticationStrategy();

  void setAuthenticationStrategy(@Nullable final AuthenticationStrategy authenticationStrategy);

  Boolean getNormalizeUri();

  void setNormalizeUri(final Boolean normalizeUri);

  Boolean getDisableContentCompression();

  void setDisableContentCompression(final Boolean disableContentCompression);

  HttpClientConfiguration copy();
}
