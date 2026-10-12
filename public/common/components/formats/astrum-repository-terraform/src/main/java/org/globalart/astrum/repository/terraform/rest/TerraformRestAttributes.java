package org.globalart.astrum.repository.terraform.rest;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig;
import org.globalart.astrum.rest.ValidationErrorsException;

import static org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig.REGISTRY_HOST;
import static org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig.REQUIRE_AUTHENTICATION;
import static org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig.TERRAFORM;
import static org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig.UPSTREAM_HOST;

/**
 * Moves the {@code terraform} object of the REST API into a repository configuration and back.
 */
final class TerraformRestAttributes
{
  private TerraformRestAttributes() {
  }

  static TerraformAttributes of(final Repository repository) {
    return new TerraformAttributes(TerraformRepositoryConfig.requireAuthentication(repository),
        TerraformRepositoryConfig.registryHost(repository.getConfiguration()).orElse(null),
        TerraformRepositoryConfig.upstreamHost(repository.getConfiguration()).orElse(null));
  }

  /**
   * @throws ValidationErrorsException when the registry host is no host name, or another Terraform repository
   *                                   already answers for it
   */
  static Configuration apply(
      final Configuration configuration,
      @Nullable final TerraformAttributes terraform,
      final RepositoryManager repositoryManager)
  {
    NestedAttributesMap attributes = configuration.attributes(TERRAFORM);
    attributes.set(REQUIRE_AUTHENTICATION,
        terraform != null && Boolean.TRUE.equals(terraform.getRequireAuthentication()));
    applyUpstreamHost(attributes, terraform == null ? null : terraform.getUpstreamHost());
    String requested = terraform == null ? null : terraform.getRegistryHost();
    if (requested == null || requested.isBlank()) {
      attributes.remove(REGISTRY_HOST);
      return configuration;
    }
    Optional<String> host = TerraformRepositoryConfig.normalizeHost(requested);
    if (host.isEmpty()) {
      throw new ValidationErrorsException("terraform." + REGISTRY_HOST,
          "'" + requested.strip() + "' is not a host name, optionally followed by :port");
    }
    for (Repository other : repositoryManager.browse()) {
      if (TerraformFormat.NAME.equals(other.getFormat().getValue())
          && !other.getName().equals(configuration.getRepositoryName())
          && host.equals(TerraformRepositoryConfig.registryHost(other.getConfiguration()))) {
        throw new ValidationErrorsException("terraform." + REGISTRY_HOST,
            "Repository " + other.getName() + " already answers for " + host.get());
      }
    }
    attributes.set(REGISTRY_HOST, host.get());
    return configuration;
  }

  private static void applyUpstreamHost(final NestedAttributesMap attributes, @Nullable final String requested) {
    if (requested == null || requested.isBlank()) {
      attributes.remove(UPSTREAM_HOST);
      return;
    }
    String host = TerraformRepositoryConfig.normalizeHost(requested).orElseThrow(() ->
        new ValidationErrorsException("terraform." + UPSTREAM_HOST,
            "'" + requested.strip() + "' is not a host name, optionally followed by :port"));
    attributes.set(UPSTREAM_HOST, host);
  }
}
