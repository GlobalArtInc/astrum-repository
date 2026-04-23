package org.globalart.astrum.repository.alpine.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.GroupRepositoryApiRequestToConfigurationConverter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class AlpineGroupRepositoryApiRequestToConfigurationConverter
    extends GroupRepositoryApiRequestToConfigurationConverter<AlpineGroupRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public AlpineGroupRepositoryApiRequestToConfigurationConverter(@Lazy final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final AlpineGroupRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    AlpineSigningAttributes.apply(configuration, request.getAlpineSigning(), repositoryManager.get(request.getName()));
    return configuration;
  }
}
