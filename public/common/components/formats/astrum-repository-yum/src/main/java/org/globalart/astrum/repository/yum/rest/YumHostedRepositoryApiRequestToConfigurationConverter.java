package org.globalart.astrum.repository.yum.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class YumHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<YumHostedRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public YumHostedRepositoryApiRequestToConfigurationConverter(@Lazy final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final YumHostedRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    YumAttributes.apply(configuration, request.getYum());
    YumSigningAttributes.apply(configuration, request.getYumSigning(), repositoryManager.get(request.getName()));
    return configuration;
  }
}
