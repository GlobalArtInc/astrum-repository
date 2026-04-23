package org.globalart.astrum.repository.alpine.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class AlpineHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<AlpineHostedRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public AlpineHostedRepositoryApiRequestToConfigurationConverter(@Lazy final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final AlpineHostedRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    AlpineSigningAttributes.apply(configuration, request.getAlpineSigning(), repositoryManager.get(request.getName()));
    return configuration;
  }
}
