
package org.globalart.astrum.repository.webhooks;

import java.net.URI;
import java.util.List;
import java.util.Map;
import javax.annotation.Nullable;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.repository.config.RepositoryName;
import org.globalart.astrum.validation.constraint.Url;
import org.globalart.astrum.validation.constraint.SsrfSafeUrl;

import com.google.common.base.Splitter;
import com.google.common.base.Strings;

public class RepositoryWebhookCapabilityConfiguration
    extends CapabilityConfigurationSupport
    implements RepositoryWebhook.Configuration
{
  static final String P_REPOSITORY = "repository";

  static final String P_NAMES = "names";

  static final String P_URL = "url";

  static final String P_SECRET = "secret";

  @RepositoryName
  String repository;

  @RepositoryWebhookType
  List<String> names;

  @SsrfSafeUrl
  @Url
  URI url;

  @Nullable
  private final String secretId;

  private final SecretsService secretsService;

  RepositoryWebhookCapabilityConfiguration(
      final Map<String, String> properties,
      final SecretsService secretsService)
  {
    repository = properties.get(P_REPOSITORY);
    names = parseList(properties.get(P_NAMES));
    url = parseUri(properties.get(P_URL));
    // Store only the secret ID - decrypt on-demand when needed
    this.secretId = Strings.emptyToNull(properties.get(P_SECRET));
    this.secretsService = secretsService;
  }

  private static final Splitter LIST_SPLITTER = Splitter.on(',').trimResults().omitEmptyStrings();

  private static List<String> parseList(final String value) {
    return LIST_SPLITTER.splitToList(value);
  }

  @Override
  public String getRepository() {
    return repository;
  }

  public List<String> getNames() {
    return names;
  }

  @Override
  public URI getUrl() {
    return url;
  }

  @Override
  @Nullable
  public String getSecret() {
    return decryptSecret(secretId, secretsService);
  }
}
