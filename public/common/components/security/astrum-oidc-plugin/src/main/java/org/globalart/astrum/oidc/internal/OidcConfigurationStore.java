package org.globalart.astrum.oidc.internal;

import java.util.Optional;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class OidcConfigurationStore
{
  static final String KEY = "astrum.oidc.configuration";

  static final String SECRET_PURPOSE = "oidc-client-secret";

  private final GlobalKeyValueStore keyValueStore;

  private final SecretsService secretsService;

  private volatile Optional<OidcConfiguration> cached;

  @Autowired
  public OidcConfigurationStore(final GlobalKeyValueStore keyValueStore, final SecretsService secretsService) {
    this.keyValueStore = checkNotNull(keyValueStore);
    this.secretsService = checkNotNull(secretsService);
  }

  public Optional<OidcConfiguration> load() {
    Optional<OidcConfiguration> current = cached;
    if (current == null) {
      current = keyValueStore.get(KEY, OidcConfiguration.class);
      cached = current;
    }
    return current;
  }

  public boolean isConfigured() {
    return load().isPresent();
  }

  public record SecretChange(boolean replace, String value)
  {
    public static SecretChange keep() {
      return new SecretChange(false, null);
    }

    public static SecretChange remove() {
      return new SecretChange(true, null);
    }

    public static SecretChange set(final String value) {
      return new SecretChange(true, value);
    }
  }

  public synchronized void save(final OidcConfiguration configuration, final SecretChange secret, final String userId) {
    String previousSecretId = load().map(OidcConfiguration::getClientSecretId).orElse(null);
    if (!secret.replace()) {
      configuration.setClientSecretId(previousSecretId);
    }
    else if (secret.value() != null) {
      Secret encrypted = secretsService.encrypt(SECRET_PURPOSE, secret.value().toCharArray(), userId);
      configuration.setClientSecretId(encrypted.getId());
    }
    else {
      configuration.setClientSecretId(null);
    }
    keyValueStore.setString(KEY, (Object) configuration);
    cached = Optional.of(configuration);
    if (secret.replace() && previousSecretId != null) {
      secretsService.remove(secretsService.from(previousSecretId));
    }
  }

  public char[] clientSecret(final OidcConfiguration configuration) {
    if (configuration.getClientSecretId() == null) {
      return null;
    }
    return secretsService.from(configuration.getClientSecretId()).decrypt();
  }
}
