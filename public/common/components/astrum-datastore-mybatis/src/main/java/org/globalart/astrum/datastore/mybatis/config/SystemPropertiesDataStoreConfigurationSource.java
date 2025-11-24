package org.globalart.astrum.datastore.mybatis.config;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

@Component
@Qualifier(SystemPropertiesDataStoreConfigurationSource.NAME)
@Order(SystemPropertiesDataStoreConfigurationSource.ORDER_SYSTEM_PROPERTIES)
public class SystemPropertiesDataStoreConfigurationSource
    extends ExternalDataStoreConfigurationSource
{
  static final String NAME = "properties";

  @Override
  public String getName() {
    return "System properties";
  }

  @Override
  protected String qualifier() {
    return NAME;
  }

  @Override
  protected String lookup(final String storeName, final String attribute) {
    return System.getProperty(propertyKey(storeName, attribute));
  }
}
