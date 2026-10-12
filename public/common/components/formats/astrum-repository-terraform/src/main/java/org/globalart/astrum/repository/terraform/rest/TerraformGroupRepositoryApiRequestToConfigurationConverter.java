package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.GroupRepositoryApiRequestToConfigurationConverter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class TerraformGroupRepositoryApiRequestToConfigurationConverter
    extends GroupRepositoryApiRequestToConfigurationConverter<TerraformGroupRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public TerraformGroupRepositoryApiRequestToConfigurationConverter(@Lazy final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final TerraformGroupRepositoryApiRequest request) {
    return TerraformRestAttributes.apply(super.convert(request), request.getTerraform(), repositoryManager);
  }
}
