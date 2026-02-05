
package org.globalart.astrum.repository.content.fluent.constraints;

import java.util.HashSet;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.ContentRepository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.group.GroupFacet;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toSet;

/**
 * By default, when a group repository is used in Fluent queries, only the group repository storage is considered,
 * you can use this constraint to alter that functionality if necessary
 */
public class GroupRepositoryConstraint
    implements FluentQueryConstraint
{
  private final GroupRepositoryLocation groupRepositoryLocation;

  public GroupRepositoryConstraint(final GroupRepositoryLocation groupRepositoryLocation) {
    this.groupRepositoryLocation = checkNotNull(groupRepositoryLocation);
  }

  @Override
  public Set<Integer> getRepositoryIds(final Repository repository) {
    Set<Integer> repositoryIds = new HashSet<>();
    switch (groupRepositoryLocation) {
      case LOCAL:
        repositoryIds.add(repository.facet(ContentFacet.class).contentRepositoryId());
        break;
      case MEMBERS:
        repositoryIds.addAll(getLeafRepositoryIds(repository));
        break;
      case BOTH:
        repositoryIds.add(repository.facet(ContentFacet.class).contentRepositoryId());
        repositoryIds.addAll(getLeafRepositoryIds(repository));
        break;
    }
    return repositoryIds;
  }

  private Set<Integer> getLeafRepositoryIds(final Repository repository) {
    return repository.facet(GroupFacet.class)
        .leafMembers()
        .stream()
        .map(leafRepository -> leafRepository.facet(ContentFacet.class))
        .map(ContentRepository::contentRepositoryId)
        .collect(toSet());
  }

  /**
   * How to interact with assets in group repositories
   */
  public enum GroupRepositoryLocation
  {
    /**
     * Only look for assets in the group storage, ignoring member content
     */
    LOCAL,
    /**
     * Only look for assets in the member repository storage, ignoring group content
     */
    MEMBERS,
    /**
     * Look for assets in both group repository storage and member repository storage
     */
    BOTH
  }

  public static GroupRepositoryConstraint of(final GroupRepositoryLocation groupRepositoryLocation) {
    return new GroupRepositoryConstraint(groupRepositoryLocation);
  }
}
