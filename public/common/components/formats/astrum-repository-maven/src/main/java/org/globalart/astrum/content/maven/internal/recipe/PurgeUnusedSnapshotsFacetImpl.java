
package org.globalart.astrum.content.maven.internal.recipe;

import java.time.LocalDate;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.content.maven.store.Maven2ComponentStore;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.group.MavenGroupFacet;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.globalart.astrum.scheduling.TaskInterruptedException;

import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Implementation of {@link PurgeUnusedSnapshotsFacet} for the NewDB. The implementation assumes that this facet will
 * only be used with hosted and group repositories.
 *
 * @since 3.30
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PurgeUnusedSnapshotsFacetImpl
    extends FacetSupport
    implements PurgeUnusedSnapshotsFacet
{

  private final Type groupType;

  private final Type hostedType;

  private final int findUnusedLimit;

  @Autowired
  public PurgeUnusedSnapshotsFacetImpl(
      @Qualifier(GroupType.NAME) final Type groupType,
      @Qualifier(HostedType.NAME) final Type hostedType,
      @Value("${astrum.tasks.purgeUnusedSnapshots.findUnusedLimit:100}") final int findUnusedLimit)
  {
    this.groupType = checkNotNull(groupType);
    this.hostedType = checkNotNull(hostedType);
    this.findUnusedLimit = findUnusedLimit;
    checkArgument(findUnusedLimit >= 10 && findUnusedLimit <= 1000,
        "astrum.tasks.purgeUnusedSnapshots.findUnusedLimit must be between 10 and 1000");
  }

  @Override
  @Guarded(by = STARTED)
  public void purgeUnusedSnapshots(final int numberOfDays) {
    checkArgument(numberOfDays > 0, "Number of days must be greater than zero");
    log.info("Purging unused snapshots {} days or older from repository {}", numberOfDays, getRepository().getName());
    if (groupType.equals(getRepository().getType())) {
      processAsGroup(facet(MavenGroupFacet.class), numberOfDays);
    }
    else if (hostedType.equals(getRepository().getType())) {
      purgeSnapshotsFromRepository(numberOfDays);
    }
    else {
      log.debug("Skipping repository {}, is not group or hosted", getRepository().getName());
    }
  }

  /**
   * Processes this facet's associated repository as a group repository, iterating over its members.
   */
  private void processAsGroup(final MavenGroupFacet groupFacet, final int numberOfDays) {
    groupFacet.leafMembers()
        .stream()
        .filter(member -> hostedType.equals(member.getType()))
        .forEach(member -> member.facet(PurgeUnusedSnapshotsFacet.class).purgeUnusedSnapshots(numberOfDays));
  }

  /**
   * Purges snapshots from the given repository.
   */
  private void purgeSnapshotsFromRepository(final int numberOfDays) {
    LocalDate olderThan = LocalDate.now().minusDays(numberOfDays);
    deleteUnusedSnapshotComponents(olderThan);
  }

  /**
   * Deletes the unused snapshot components and their associated assets and metadata.
   */
  private void deleteUnusedSnapshotComponents(final LocalDate olderThan) {
    Repository repository = getRepository();
    MavenContentFacetImpl contentFacet = (MavenContentFacetImpl) repository.facet(MavenContentFacet.class);
    Maven2ComponentStore componentStore = (Maven2ComponentStore) contentFacet.stores().componentStore;

    // totalComponents is used just for the reporting process
    long totalComponents = contentFacet.components().count();
    log.info("Found {} total components in repository {} to evaluate for unused snapshots", totalComponents,
        repository.getName());

    while (!isCanceled()) {

      // During every new iteration, first components are already removed, so no offset needed
      int[] componentIds = componentStore
          .selectUnusedSnapshots(contentFacet.contentRepositoryId(), olderThan, findUnusedLimit)
          .stream()
          .mapToInt(id -> id)
          .toArray();

      if (componentIds.length == 0) {
        return;
      }

      contentFacet.deleteComponents(componentIds);
    }
  }

  private boolean isCanceled() {
    try {
      CancelableHelper.checkCancellation();
      return false;
    }
    catch (TaskInterruptedException e) { // NOSONAR
      log.warn("Purge unused Maven snapshots job is canceled");
      return true;
    }
  }
}
