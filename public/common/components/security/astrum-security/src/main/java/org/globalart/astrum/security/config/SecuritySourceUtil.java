

package org.globalart.astrum.security.config;

import java.util.Set;

import static com.google.common.collect.Sets.newHashSet;

/**
 * @since 3.20
 */
public class SecuritySourceUtil
{
  private static final Set<String> CASE_INSENSITIVE_SOURCES = newHashSet("crowd", "ldap");

  public static boolean isCaseInsensitiveSource(final String source) {
    return CASE_INSENSITIVE_SOURCES.contains(source.toLowerCase());
  }
}
