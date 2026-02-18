
package org.globalart.astrum.repository.group;

import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Content;

/**
 * Group facet.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface GroupFacet
    extends Facet
{
  /**
   * Check if given repository is a member of the group.
   */
  boolean member(String repositoryName);

  /**
   * Check if given repository is a member of the group.
   */
  boolean member(Repository repository);

  /**
   * Return list of all member repositories including transitive
   *
   * @since 3.6.1
   */
  List<Repository> allMembers();

  /**
   * Return list of (non-transitive) member repositories.
   */
  List<Repository> members();

  /**
   * Return the full list of members, including the members of groups, but excluding groups.
   */
  List<Repository> leafMembers();

  /**
   * Removes all entries from the group cache and the member caches.
   */
  void invalidateGroupCaches();

  /**
   * Returns {@code true} if the content is considered stale; otherwise {@code false}.
   */
  boolean isStale(@Nullable final Content content);

  /**
   * Maintains the latest cache information in the given content's attributes.
   */
  void maintainCacheInfo(final AttributesMap attributesMap);
}
