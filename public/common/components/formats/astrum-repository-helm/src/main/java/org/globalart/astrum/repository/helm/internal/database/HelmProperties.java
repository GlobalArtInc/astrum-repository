
package org.globalart.astrum.repository.helm.internal.database;

import java.util.Arrays;
import java.util.Optional;

/**
 * Database property names for Helm asset attributes
 *
 * @since 3.28
 */
public enum HelmProperties
{
  DESCRIPTION("description"),
  ENGINE("engine"),
  HOME("home"),
  ICON("icon"),
  APP_VERSION("appVersion"),
  KEYWORDS("keywords"),
  MAINTAINERS("maintainers"),
  NAME("name"),
  SOURCES("sources"),
  VERSION("version");

  private String propertyName;

  HelmProperties(final String type) {
    this.propertyName = type;
  }

  public String getPropertyName() {
    return propertyName;
  }

  public static Optional<HelmProperties> findByPropertyName(String propertyName) {
    return Arrays.stream(HelmProperties.values())
        .filter(properties -> propertyName.equals(properties.getPropertyName()))
        .findAny();
  }
}
