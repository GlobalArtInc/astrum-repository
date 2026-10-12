package org.globalart.astrum.repository.yum.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.GroupRepositoryApiRequestToConfigurationConverter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class YumGroupRepositoryApiRequestToConfigurationConverter
    extends GroupRepositoryApiRequestToConfigurationConverter<YumGroupRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public YumGroupRepositoryApiRequestToConfigurationConverter(@Lazy final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final YumGroupRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    YumSigningAttributes.apply(configuration, request.getYumSigning(), repositoryManager.get(request.getName()));
    return configuration;
  }
}
