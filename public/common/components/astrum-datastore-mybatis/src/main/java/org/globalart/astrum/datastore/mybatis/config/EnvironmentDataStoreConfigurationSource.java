package org.globalart.astrum.datastore.mybatis.config;

import java.util.Map;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

@Component
@Qualifier(EnvironmentDataStoreConfigurationSource.NAME)
@Order(EnvironmentDataStoreConfigurationSource.ORDER_ENVIRONMENT_VALUES)
public class EnvironmentDataStoreConfigurationSource
    extends ExternalDataStoreConfigurationSource
{
  static final String NAME = "environment";

  private final Map<String, String> environment;

  public EnvironmentDataStoreConfigurationSource() {
    this(System.getenv());
  }

  EnvironmentDataStoreConfigurationSource(final Map<String, String> environment) {
    this.environment = environment;
  }

  @Override
  public String getName() {
    return "Environment variables";
  }

  @Override
  protected String qualifier() {
    return NAME;
  }

  @Override
  protected String lookup(final String storeName, final String attribute) {
    return environment.get(environmentKey(storeName, attribute));
  }
}
