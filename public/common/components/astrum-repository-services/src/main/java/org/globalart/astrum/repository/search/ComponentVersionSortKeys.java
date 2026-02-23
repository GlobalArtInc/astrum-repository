
package org.globalart.astrum.repository.search;

import java.util.Set;

/**
 * Canonical sort keys for the version-browsing API.
 *
 * <p>
 * This is the single source of truth. The resource layer validates against these keys,
 * and the SQL layer maps from them to SQL expressions.
 */
public final class ComponentVersionSortKeys
{
  private ComponentVersionSortKeys() {
  }

  public static final String VERSION = "version";

  public static final String LAST_UPDATED = "lastUpdated";

  public static final String REPOSITORIES = "repositories";

  public static final Set<String> ALL = Set.of(VERSION, LAST_UPDATED, REPOSITORIES);
}
