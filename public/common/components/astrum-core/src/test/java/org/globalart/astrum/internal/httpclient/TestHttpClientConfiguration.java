
package org.globalart.astrum.internal.httpclient;

import org.apache.http.client.AuthenticationStrategy;
import org.apache.http.client.RedirectStrategy;
import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.ConnectionConfiguration;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;
import org.globalart.astrum.httpclient.config.ProxyConfiguration;

import java.util.Optional;

public class TestHttpClientConfiguration
    implements HttpClientConfiguration
{

  private ConnectionConfiguration connection;

  private ProxyConfiguration proxy;

  private RedirectStrategy redirectStrategy;

  private AuthenticationConfiguration authentication;

  private AuthenticationStrategy authenticationStrategy;

  private Boolean normalizeUri;

  private Boolean disableContentCompression;

  public TestHttpClientConfiguration() {

  }

  public TestHttpClientConfiguration(ProxyConfiguration proxy) {
    this.proxy = proxy;
  }

  @Override
  public ConnectionConfiguration getConnection() {
    return connection;
  }

  public void setConnection(ConnectionConfiguration connection) {
    this.connection = connection;
  }

  @Override
  public ProxyConfiguration getProxy() {
    return proxy;
  }

  public void setProxy(ProxyConfiguration proxy) {
    this.proxy = proxy;
  }

  @Override
  public RedirectStrategy getRedirectStrategy() {
    return redirectStrategy;
  }

  public void setRedirectStrategy(RedirectStrategy redirectStrategy) {
    this.redirectStrategy = redirectStrategy;
  }

  @Override
  public AuthenticationConfiguration getAuthentication() {
    return authentication;
  }

  public void setAuthentication(AuthenticationConfiguration authentication) {
    this.authentication = authentication;
  }

  @Override
  public AuthenticationStrategy getAuthenticationStrategy() {
    return authenticationStrategy;
  }

  public void setAuthenticationStrategy(AuthenticationStrategy authenticationStrategy) {
    this.authenticationStrategy = authenticationStrategy;
  }

  @Override
  public Boolean getNormalizeUri() {
    return Optional.ofNullable(normalizeUri).orElse(false);
  }

  public void setNormalizeUri(Boolean normalizeUri) {
    this.normalizeUri = normalizeUri;
  }

  @Override
  public Boolean getDisableContentCompression() {
    return disableContentCompression;
  }

  public void setDisableContentCompression(Boolean disableContentCompression) {
    this.disableContentCompression = disableContentCompression;
  }

  @Override
  public TestHttpClientConfiguration copy() {
    return this;
  }
}
