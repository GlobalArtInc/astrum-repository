
package org.globalart.astrum.repository.content.fluent.internal;

import java.util.Collection;
import java.util.List;
import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.constraints.FluentQueryConstraint;
import org.globalart.astrum.repository.types.GroupType;

import static java.util.Collections.singleton;
import static java.util.stream.Collectors.toSet;

/**
 * Utility methods used by both {@link FluentAssetsImpl} and {@link FluentComponentsImpl}
 *
 * @since 3.27
 */
final class RepositoryContentUtil
{
  private RepositoryContentUtil() {
  }

  static boolean isGroupRepository(final Repository repository) {
    return GroupType.NAME.equals(repository.getType().getValue());
  }

  static Set<Integer> getRepositoryIds(
      @Nullable final List<FluentQueryConstraint> constraints,
      final ContentFacet contentFacet,
      final Repository repository)
  {
    // no constraints supplied, just use the repository of the contentFacet supplied
    if (constraints == null || constraints.isEmpty()) {
      return singleton(contentFacet.contentRepositoryId());
    }

    Set<Integer> repositoryIds = constraints.stream()
        .map(constraint -> constraint.getRepositoryIds(repository))
        .flatMap(Collection::stream)
        .collect(toSet());

    constraints.forEach(constraint -> repositoryIds.addAll(constraint.getRepositoryIds(repository)));

    // if we get to this point and no repository has been selected based on constraints, fallback to the repository
    // of the contentFacet supplied
    if (repositoryIds.isEmpty()) {
      repositoryIds.add(contentFacet.contentRepositoryId());
    }

    return repositoryIds;
  }
}
