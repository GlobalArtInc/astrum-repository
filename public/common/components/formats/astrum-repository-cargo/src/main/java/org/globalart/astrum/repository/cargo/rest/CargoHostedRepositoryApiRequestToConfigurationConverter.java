package org.globalart.astrum.repository.cargo.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;
import org.springframework.stereotype.Component;

@Component
public class CargoHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<CargoHostedRepositoryApiRequest>
{
  @Override
  public Configuration convert(final CargoHostedRepositoryApiRequest request) {
    return CargoRestAttributes.apply(super.convert(request), request.getCargo());
  }
}
