package org.globalart.astrum.repository.cargo.internal;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.security.anonymous.AnonymousHelper;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;

public final class CargoRepositoryConfig
{
  public static final String CARGO = "cargo";

  public static final String REQUIRE_AUTHENTICATION = "requireAuthentication";

  private CargoRepositoryConfig() {
  }

  public static boolean requireAuthentication(final Repository repository) {
    return requireAuthentication(repository.getConfiguration());
  }

  public static boolean requireAuthentication(final Configuration configuration) {
    if (configuration == null) {
      return false;
    }
    NestedAttributesMap cargo = configuration.attributes(CARGO);
    Object value = cargo.get(REQUIRE_AUTHENTICATION);
    return value instanceof Boolean flag ? flag : Boolean.parseBoolean(String.valueOf(value));
  }

  public static boolean isAnonymous() {
    Subject subject = SecurityUtils.getSubject();
    return subject == null || subject.getPrincipal() == null || AnonymousHelper.isAnonymous(subject);
  }
}
