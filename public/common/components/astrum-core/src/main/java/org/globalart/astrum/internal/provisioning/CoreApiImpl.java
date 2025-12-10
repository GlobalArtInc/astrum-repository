
package org.globalart.astrum.internal.provisioning;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.CoreApi;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.common.entity.EntityHelper;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.ConnectionConfiguration;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;
import org.globalart.astrum.httpclient.config.NtlmAuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.ProxyConfiguration;
import org.globalart.astrum.httpclient.config.ProxyServerConfiguration;
import org.globalart.astrum.httpclient.config.UsernameAuthenticationConfiguration;
import org.globalart.astrum.internal.app.BaseUrlCapabilityDescriptor;
import org.globalart.astrum.internal.capability.DefaultCapabilityReference;
import org.globalart.astrum.security.UserIdHelper;

import com.google.common.collect.ImmutableMap;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.httpclient.config.AuthenticationConfiguration.AUTHENTICATION_CONFIGURATION;
import org.springframework.stereotype.Component;

/**
 * @since 3.0
 */
@Component
public class CoreApiImpl
    implements CoreApi
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final CapabilityRegistry capabilityRegistry;

  private final HttpClientManager httpClientManager;

  private final SecretsService secretsService;

  @Autowired
  public CoreApiImpl(
      final CapabilityRegistry capabilityRegistry,
      final HttpClientManager httpClientManager,
      final SecretsService secretsService)
  {
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
    this.httpClientManager = checkNotNull(httpClientManager);
    this.secretsService = checkNotNull(secretsService);
  }

  @Override
  public void baseUrl(final String url) {
    checkNotNull(url);
    DefaultCapabilityReference existing = (DefaultCapabilityReference) capabilityRegistry.getAll()
        .stream()
        .filter(
            capabilityReference -> BaseUrlCapabilityDescriptor.TYPE.equals(
                capabilityReference.context().descriptor().type()))
        .findFirst()
        .orElse(null);

    if (existing != null) {
      log.info("BaseUrl capability updated to: {}",
          capabilityRegistry.update(existing.id(), existing.isActive(), existing.notes(), ImmutableMap.of("url", url)));
    }
    else {
      log.info("BaseUrl capability created as: {}",
          capabilityRegistry.add(BaseUrlCapabilityDescriptor.TYPE, true, "configured through api",
              ImmutableMap.of("url", url)));
    }
  }

  @Override
  public void removeBaseUrl() {
    DefaultCapabilityReference existing = (DefaultCapabilityReference) capabilityRegistry.getAll()
        .stream()
        .filter(
            capabilityReference -> BaseUrlCapabilityDescriptor.TYPE.equals(
                capabilityReference.context().descriptor().type()))
        .findFirst()
        .orElse(null);

    if (existing != null) {
      log.info("Deleting BaseUrl capability");
      capabilityRegistry.remove(existing.id());
    }
    else {
      log.info("No BaseUrl capability configured to remove");
    }
  }

  @Override
  public void userAgentCustomization(final String userAgentSuffix) {
    checkNotNull(userAgentSuffix);
    HttpClientConfiguration configuration = detachedConfiguration();
    connection(configuration).setUserAgentSuffix(userAgentSuffix);
    httpClientManager.setConfiguration(configuration);
  }

  @Override
  public void connectionTimeout(final int timeout) {
    checkArgument((timeout >= 1 && timeout <= 3600), "Value must be between 1 and 3600");
    HttpClientConfiguration configuration = detachedConfiguration();
    connection(configuration).setTimeout(Time.seconds(timeout));
    httpClientManager.setConfiguration(configuration);
  }

  @Override
  public void connectionRetryAttempts(final int retries) {
    checkArgument((retries >= 0 && retries <= 10), "Value must be between 0 and 10");
    HttpClientConfiguration configuration = detachedConfiguration();
    connection(configuration).setRetries(retries);
    httpClientManager.setConfiguration(configuration);
  }

  @Override
  public void httpProxy(final String host, final int port) {
    checkNotNull(host);
    checkPort(port);
    configureProxy(host, port, false, null, null);
  }

  @Override
  public void httpProxyWithBasicAuth(final String host, final int port, final String username, final String password) {
    checkNotNull(host);
    checkPort(port);
    checkNotNull(username);
    checkNotNull(password);
    UsernameAuthenticationConfiguration config = new UsernameAuthenticationConfiguration();
    config.setUsername(username);
    final Secret secret = encrypt(password);
    config.setPassword(secret);
    configureProxy(host, port, false, config, secret);
  }

  private Secret encrypt(final String password) {
    return secretsService.encryptMaven(AUTHENTICATION_CONFIGURATION, password.toCharArray(), UserIdHelper.get());
  }

  @Override
  public void httpProxyWithNTLMAuth(
      final String host,
      final int port,
      final String username,
      final String password,
      final String ntlmHost,
      final String domain)
  {
    checkNotNull(host);
    checkPort(port);
    checkNotNull(username);
    checkNotNull(password);
    checkNotNull(ntlmHost);
    checkNotNull(domain);
    NtlmAuthenticationConfiguration config = new NtlmAuthenticationConfiguration();
    config.setUsername(username);
    final Secret secret = encrypt(password);
    config.setPassword(secret);
    config.setHost(ntlmHost);
    config.setDomain(domain);
    configureProxy(host, port, false, config, secret);
  }

  @Override
  public void removeHTTPProxy() {
    HttpClientConfiguration configuration = detachedConfiguration();
    if (configuration.getProxy() != null) {
      configuration.setProxy(null);
      httpClientManager.setConfiguration(configuration);
    }
  }

  @Override
  public void httpsProxy(final String host, final int port) {
    checkNotNull(host);
    checkPort(port);
    configureProxy(host, port, true, null, null);
  }

  @Override
  public void httpsProxyWithBasicAuth(final String host, final int port, final String username, final String password) {
    checkNotNull(host);
    checkPort(port);
    checkNotNull(username);
    checkNotNull(password);
    UsernameAuthenticationConfiguration config = new UsernameAuthenticationConfiguration();
    config.setUsername(username);
    final Secret secret = encrypt(password);
    config.setPassword(secret);
    configureProxy(host, port, true, config, secret);
  }

  @Override
  public void httpsProxyWithNTLMAuth(
      final String host,
      final int port,
      final String username,
      final String password,
      final String ntlmHost,
      final String domain)
  {
    checkNotNull(host);
    checkPort(port);
    checkNotNull(username);
    checkNotNull(password);
    checkNotNull(ntlmHost);
    checkNotNull(domain);
    NtlmAuthenticationConfiguration config = new NtlmAuthenticationConfiguration();
    config.setUsername(username);
    final Secret secret = encrypt(password);
    config.setPassword(secret);
    config.setHost(ntlmHost);
    config.setDomain(domain);
    configureProxy(host, port, true, config, secret);
  }

  @Override
  public void removeHTTPSProxy() {
    HttpClientConfiguration configuration = detachedConfiguration();
    if (configuration.getProxy() != null) {
      configuration.getProxy().setHttps(null);
      httpClientManager.setConfiguration(configuration);
    }
  }

  @Override
  public void nonProxyHosts(final String... nonProxyHosts) {
    HttpClientConfiguration configuration = detachedConfiguration();
    if (configuration.getProxy() == null) {
      throw new IllegalStateException("Cannot configure non-proxy hosts without a configured proxy");
    }
    configuration.getProxy().setNonProxyHosts(nonProxyHosts);
    httpClientManager.setConfiguration(configuration);
  }

  private ConnectionConfiguration connection(final HttpClientConfiguration configuration) {
    if (configuration.getConnection() == null) {
      configuration.setConnection(new ConnectionConfiguration());
    }
    return configuration.getConnection();
  }

  private ProxyConfiguration proxy(final HttpClientConfiguration configuration) {
    if (configuration.getProxy() == null) {
      configuration.setProxy(new ProxyConfiguration());
    }
    return configuration.getProxy();
  }

  private void configureProxy(
      final String host,
      final int port,
      final boolean https,
      @Nullable final AuthenticationConfiguration auth,
      @Nullable final Secret secret)
  {
    HttpClientConfiguration configuration = detachedConfiguration();
    if (https && (configuration.getProxy() == null || configuration.getProxy().getHttp() == null)) {
      throw new IllegalStateException("Cannot configure https proxy without http proxy");
    }
    ProxyServerConfiguration proxyServerConfiguration = new ProxyServerConfiguration();
    proxyServerConfiguration.setEnabled(true);
    proxyServerConfiguration.setHost(host);
    proxyServerConfiguration.setPort(port);
    proxyServerConfiguration.setAuthentication(auth);
    if (https) {
      proxy(configuration).setHttps(proxyServerConfiguration);
    }
    else {
      proxy(configuration).setHttp(proxyServerConfiguration);
    }
    try {
      httpClientManager.setConfiguration(configuration);
    }
    catch (Exception e) {
      secretsService.remove(secret);
      throw e;
    }
  }

  /**
   * Must use a 'detached' copy otherwise we can't serialize the singleton configuration back to the db.
   */
  private HttpClientConfiguration detachedConfiguration() {
    HttpClientConfiguration configuration = httpClientManager.getConfiguration();
    EntityHelper.clearMetadata(configuration);
    return configuration;
  }

  private void checkPort(final int port) {
    checkArgument(port >= 1 && port <= 65535, "Port must be between 1 and 65535");
  }
}
