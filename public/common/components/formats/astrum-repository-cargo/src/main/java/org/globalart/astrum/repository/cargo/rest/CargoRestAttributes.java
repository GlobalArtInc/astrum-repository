package org.globalart.astrum.repository.cargo.rest;

import org.globalart.astrum.repository.cargo.internal.CargoRepositoryConfig;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.model.CargoAttributes;

final class CargoRestAttributes
{
  private CargoRestAttributes() {
  }

  static Configuration apply(final Configuration configuration, final CargoAttributes cargo) {
    boolean requireAuthentication = cargo != null && Boolean.TRUE.equals(cargo.getRequireAuthentication());
    configuration.attributes(CargoRepositoryConfig.CARGO)
        .set(CargoRepositoryConfig.REQUIRE_AUTHENTICATION, requireAuthentication);
    return configuration;
  }
}
