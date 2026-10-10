package org.globalart.astrum.ldap.internal;

import java.util.Locale;

final class LdapEnums
{
  private LdapEnums() {
  }

  static <E extends Enum<E>> E parse(final Class<E> type, final String value) {
    if (value == null || value.isBlank()) {
      return null;
    }
    String normalized = value.trim().replace('-', '_').toUpperCase(Locale.ENGLISH);
    for (E constant : type.getEnumConstants()) {
      if (constant.name().equals(normalized)) {
        return constant;
      }
    }
    return null;
  }
}
