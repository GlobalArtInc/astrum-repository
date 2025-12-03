
package org.globalart.astrum.httpclient.internal.secrets.migration;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;
import org.globalart.astrum.httpclient.config.NtlmAuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.ProxyConfiguration;
import org.globalart.astrum.httpclient.config.ProxyServerConfiguration;
import org.globalart.astrum.httpclient.config.UsernameAuthenticationConfiguration;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.globalart.astrum.security.secrets.SecretMigrationException;
import org.globalart.astrum.security.secrets.SecretsMigratorSupport;

import com.google.common.annotations.VisibleForTesting;

import static org.globalart.astrum.httpclient.config.AuthenticationConfiguration.AUTHENTICATION_CONFIGURATION;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class HttpSecretsMigrator
    extends SecretsMigratorSupport
{
  @VisibleForTesting
  static final String GLOBAL_AUTHENTICATION_CONFIGURATION = "global authentication configuration";

  @VisibleForTesting
  static final String HTTP_PROXY_CONFIGURATION = "http proxy configuration";

  @VisibleForTesting
  static final String HTTPS_PROXY_CONFIGURATION = "https proxy configuration";

  private final HttpClientManager httpClientManager;

  @Autowired
  public HttpSecretsMigrator(
      final HttpClientManager httpClientManager,
      final SecretsService secretsService)
  {
    super(secretsService);
    this.httpClientManager = httpClientManager;
  }

  @Override
  public void migrate() {
    CancelableHelper.checkCancellation();

    List<Secret> secrets = new ArrayList<>();
    HttpClientConfiguration httpClientConfiguration = httpClientManager.getConfiguration();

    if (httpClientConfiguration != null) {
      // Migrate Global auth config
      migrateSecret(httpClientConfiguration.getAuthentication(), secrets, GLOBAL_AUTHENTICATION_CONFIGURATION);

      // Migrate HTTP proxy auth config
      AuthenticationConfiguration proxyHttpConfig = Optional.ofNullable(httpClientConfiguration.getProxy())
          .map(ProxyConfiguration::getHttp)
          .map(ProxyServerConfiguration::getAuthentication)
          .orElse(null);
      migrateSecret(proxyHttpConfig, secrets, HTTP_PROXY_CONFIGURATION);

      // Migrate HTTPS proxy auth config
      AuthenticationConfiguration proxyHttpsConfig = Optional.ofNullable(httpClientConfiguration.getProxy())
          .map(ProxyConfiguration::getHttps)
          .map(ProxyServerConfiguration::getAuthentication)
          .orElse(null);
      migrateSecret(proxyHttpsConfig, secrets, HTTPS_PROXY_CONFIGURATION);

      save(httpClientConfiguration, secrets);
    }
  }

  private void migrateSecret(
      final AuthenticationConfiguration config,
      final List<Secret> secrets,
      final String context)
  {
    Secret secret = Optional.ofNullable(config)
        .map(AuthenticationConfiguration::getSecret)
        .orElse(null);

    if (secret != null && isLegacyEncryptedString(secret)) {
      Secret migratedSecret = createSecret(AUTHENTICATION_CONFIGURATION, secret, context);
      updateConfig(config, migratedSecret);
      secrets.add(migratedSecret);
    }
  }

  private void updateConfig(final AuthenticationConfiguration configuration, final Secret secret) {
    if (configuration.getClass().equals(UsernameAuthenticationConfiguration.class)) {
      ((UsernameAuthenticationConfiguration) configuration).setPassword(secret);
    }
    else if (configuration.getClass().equals(NtlmAuthenticationConfiguration.class)) {
      ((NtlmAuthenticationConfiguration) configuration).setPassword(secret);
    }
  }

  private void save(final HttpClientConfiguration configuration, final List<Secret> secrets) {
    if (secrets.isEmpty()) {
      return;
    }

    try {
      httpClientManager.setConfiguration(configuration);
    }
    catch (Exception e) {
      quietlyRemove(secrets);
      throw new SecretMigrationException("Failed to migrate HTTP client configuration", e);
    }
  }
}
