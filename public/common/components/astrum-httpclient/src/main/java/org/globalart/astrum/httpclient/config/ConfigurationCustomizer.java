
package org.globalart.astrum.httpclient.config;

import java.net.Authenticator;
import java.net.PasswordAuthentication;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.httpclient.HttpClientPlan;
import org.globalart.astrum.httpclient.PreemptiveAuthHttpRequestInterceptor;
import org.globalart.astrum.httpclient.SSLContextSelector;
import org.globalart.astrum.httpclient.internal.AstrumHttpRoutePlanner;
import org.globalart.astrum.httpclient.internal.NonProxyHostsMatcher;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.ImmutableList;
import org.apache.http.HttpHost;
import org.apache.http.auth.AuthScope;
import org.apache.http.auth.Credentials;
import org.apache.http.auth.NTCredentials;
import org.apache.http.auth.UsernamePasswordCredentials;
import org.apache.http.client.AuthenticationStrategy;
import org.apache.http.client.RedirectStrategy;
import org.apache.http.client.config.CookieSpecs;
import org.apache.http.impl.client.StandardHttpRequestRetryHandler;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static java.util.Collections.emptyList;
import static java.util.Optional.ofNullable;
import static org.apache.http.client.config.AuthSchemes.BASIC;
import static org.apache.http.client.config.AuthSchemes.DIGEST;
import static org.apache.http.client.config.AuthSchemes.NTLM;
import static org.globalart.astrum.httpclient.HttpSchemes.HTTP;
import static org.globalart.astrum.httpclient.HttpSchemes.HTTPS;

/**
 * Applies {@link HttpClientConfiguration} to {@link HttpClientPlan}.
 *
 * @since 3.0
 */
public class ConfigurationCustomizer
    implements HttpClientPlan.Customizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static {
    /**
     * Install custom {@link Authenticator} for proxy.
     */
    Authenticator.setDefault(new Authenticator()
    {
      @Override
      protected PasswordAuthentication getPasswordAuthentication() {
        if (getRequestorType() == RequestorType.PROXY) {
          String prot = getRequestingProtocol().toLowerCase();
          String host = System.getProperty(prot + ".proxyHost", "");
          String port = System.getProperty(prot + ".proxyPort", "80");
          String user = System.getProperty(prot + ".proxyUser", "");
          String password = System.getProperty(prot + ".proxyPassword", "");

          if (getRequestingHost().equalsIgnoreCase(host)) {
            if (Integer.parseInt(port) == getRequestingPort()) {
              // Seems to be OK.
              return new PasswordAuthentication(user, password.toCharArray());
            }
          }
        }
        return null;
      }
    });
  }

  private final HttpClientConfiguration configuration;

  public ConfigurationCustomizer(final HttpClientConfiguration configuration) {
    this.configuration = checkNotNull(configuration);
  }

  @Override
  public void customize(final HttpClientPlan plan) {
    checkNotNull(plan);

    if (configuration.getConnection() != null) {
      apply(configuration.getConnection(), plan);
    }
    if (configuration.getProxy() != null) {
      apply(configuration.getProxy(), plan);
    }
    if (configuration.getAuthentication() != null) {
      apply(configuration.getAuthentication(), plan, null);
    }
    if (configuration.getRedirectStrategy() != null) {
      apply(configuration.getRedirectStrategy(), plan);
    }
    if (configuration.getAuthenticationStrategy() != null) {
      apply(configuration.getAuthenticationStrategy(), plan);
    }
    if (configuration.getNormalizeUri() != null) {
      plan.getRequest().setNormalizeUri(configuration.getNormalizeUri());
    }
    if (Boolean.TRUE.equals(configuration.getDisableContentCompression())) {
      plan.getClient().disableContentCompression();
    }
  }

  /**
   * Apply selected AuthenticationStrategy to plan.
   */
  private void apply(final AuthenticationStrategy authenticationStrategy, final HttpClientPlan plan) {
    if (authenticationStrategy != null) {
      plan.getClient().setTargetAuthenticationStrategy(authenticationStrategy);
    }
  }

  @VisibleForTesting
  void setRetryHandler(final ConnectionConfiguration connection, final HttpClientPlan plan) {
    if (connection.getRetries() != null) {
      plan.getClient().setRetryHandler(new StandardHttpRequestRetryHandler(connection.getRetries(), false));
      if (log.isTraceEnabled()) {
        log.trace("Set retry handler with {} retries", connection.getRetries());
      }
    }
  }

  /**
   * Apply connection configuration to plan.
   */
  private void apply(final ConnectionConfiguration connection, final HttpClientPlan plan) {
    if (connection.getTimeout() != null) {
      int timeout = connection.getTimeout().toMillisI();
      plan.getSocket().setSoTimeout(timeout);
      plan.getRequest().setConnectTimeout(timeout);
      plan.getRequest().setSocketTimeout(timeout);
    }

    setRetryHandler(connection, plan);

    if (connection.getUserAgentSuffix() != null) {
      checkState(plan.getUserAgentBase() != null, "Default User-Agent not set");
      plan.setUserAgentSuffix(connection.getUserAgentSuffix());
    }

    if (Boolean.TRUE.equals(connection.getUseTrustStore())) {
      plan.getAttributes().put(SSLContextSelector.USE_TRUST_STORE, Boolean.TRUE);
    }

    if (Boolean.TRUE.equals(connection.getEnableCircularRedirects())) {
      plan.getRequest().setCircularRedirectsAllowed(true);
    }

    if (Boolean.TRUE.equals(connection.getEnableCookies())) {
      plan.getRequest().setCookieSpec(CookieSpecs.DEFAULT);
    }
  }

  /**
   * Apply proxy-server configuration to plan.
   */
  private void apply(final ProxyConfiguration proxy, final HttpClientPlan plan) {
    // HTTP proxy
    ProxyServerConfiguration http = proxy.getHttp();
    if (http != null && http.isEnabled()) {
      HttpHost host = new HttpHost(http.getHost(), http.getPort());
      if (http.getAuthentication() != null) {
        apply(http.getAuthentication(), plan, host);
      }
    }

    // HTTPS proxy
    ProxyServerConfiguration https = proxy.getHttps();
    if (https != null && https.isEnabled()) {
      HttpHost host = new HttpHost(https.getHost(), https.getPort());
      if (https.getAuthentication() != null) {
        apply(https.getAuthentication(), plan, host);
      }
    }
    plan.getClient().setRoutePlanner(createRoutePlanner(proxy));
  }

  /**
   * Apply redirect strategy to plan.
   */
  private void apply(final RedirectStrategy redirectStrategy, final HttpClientPlan plan) {
    plan.getClient().setRedirectStrategy(redirectStrategy);
  }

  /**
   * Creates instance of {@link AstrumHttpRoutePlanner} from passed in configuration, never {@code null}.
   */
  @VisibleForTesting
  AstrumHttpRoutePlanner createRoutePlanner(final ProxyConfiguration proxy) {
    Map<String, HttpHost> proxies = new HashMap<>(2);

    // HTTP proxy
    ProxyServerConfiguration http = proxy.getHttp();
    if (http != null && http.isEnabled()) {
      HttpHost host = new HttpHost(http.getHost(), http.getPort());
      proxies.put(HTTP, host);
      proxies.put(HTTPS, host);
    }

    // HTTPS proxy
    ProxyServerConfiguration https = proxy.getHttps();
    if (https != null && https.isEnabled()) {
      HttpHost host = new HttpHost(https.getHost(), https.getPort());
      proxies.put(HTTPS, host);
    }

    // Non-proxy hosts (Java http.nonProxyHosts formatted glob-like patterns converted to single Regexp expression)
    Pattern nonProxyPattern = NonProxyHostsMatcher.compile(proxy.getNonProxyHosts());
    return new AstrumHttpRoutePlanner(proxies, nonProxyPattern);
  }

  /**
   * Apply authentication configuration to plan.
   */
  private void apply(
      final AuthenticationConfiguration authentication,
      final HttpClientPlan plan,
      @Nullable final HttpHost proxyHost)
  {
    Credentials credentials;
    List<String> authSchemes;

    if (authentication instanceof UsernameAuthenticationConfiguration) {
      UsernameAuthenticationConfiguration auth = (UsernameAuthenticationConfiguration) authentication;
      authSchemes = ImmutableList.of(DIGEST, BASIC);
      credentials = new UsernamePasswordCredentials(auth.getUsername(),
          ofNullable(auth.getPassword()).map(Secret::decrypt).map(String::new).orElse(null));
    }
    else if (authentication instanceof NtlmAuthenticationConfiguration) {
      NtlmAuthenticationConfiguration auth = (NtlmAuthenticationConfiguration) authentication;
      authSchemes = ImmutableList.of(NTLM, DIGEST, BASIC);
      credentials = new NTCredentials(auth.getUsername(),
          ofNullable(auth.getPassword()).map(Secret::decrypt).map(String::new).orElse(null), auth.getHost(),
          auth.getDomain());
    }
    else if (authentication instanceof BearerTokenAuthenticationConfiguration) {
      credentials = null;
      authSchemes = emptyList();
    }
    else {
      throw new IllegalArgumentException("Unsupported authentication configuration: " + authentication);
    }

    if (credentials != null) {
      if (proxyHost != null) {
        plan.addCredentials(new AuthScope(proxyHost), credentials);
        plan.getRequest().setProxyPreferredAuthSchemes(authSchemes);
      }
      else {
        plan.addCredentials(AuthScope.ANY, credentials);
        plan.getRequest().setTargetPreferredAuthSchemes(authSchemes);
      }

      if (authentication.isPreemptive()) {
        plan.getClient().addInterceptorFirst(new PreemptiveAuthHttpRequestInterceptor());
      }
    }
  }
}
