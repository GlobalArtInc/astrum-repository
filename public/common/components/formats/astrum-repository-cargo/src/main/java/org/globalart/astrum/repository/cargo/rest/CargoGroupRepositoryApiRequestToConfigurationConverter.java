package org.globalart.astrum.repository.cargo.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.GroupRepositoryApiRequestToConfigurationConverter;
import org.springframework.stereotype.Component;

@Component
public class CargoGroupRepositoryApiRequestToConfigurationConverter
    extends GroupRepositoryApiRequestToConfigurationConverter<CargoGroupRepositoryApiRequest>
{
  @Override
  public Configuration convert(final CargoGroupRepositoryApiRequest request) {
    return CargoRestAttributes.apply(super.convert(request), request.getCargo());
  }
}
