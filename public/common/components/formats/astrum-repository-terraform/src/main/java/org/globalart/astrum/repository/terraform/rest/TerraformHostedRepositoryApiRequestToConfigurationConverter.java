package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class TerraformHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<TerraformHostedRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public TerraformHostedRepositoryApiRequestToConfigurationConverter(@Lazy final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final TerraformHostedRepositoryApiRequest request) {
    Configuration configuration = TerraformRestAttributes.apply(super.convert(request), request.getTerraform(),
        repositoryManager);
    TerraformSigningAttributes.apply(configuration, request.getTerraformSigning(),
        repositoryManager.get(request.getName()));
    return configuration;
  }
}
