
package org.globalart.astrum.repository.maven.tasks;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.maven.MavenFacet;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.VersionPolicy;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.scheduling.Cancelable;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Task to purge unused snapshots of the given Maven repository.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PurgeMavenUnusedSnapshotsTask
    extends RepositoryTaskSupport
    implements Cancelable
{
  public static final String LAST_USED_FIELD_ID = "lastUsed";

  private static final String MAVEN = "maven";

  private static final String VERSION_POLICY = "versionPolicy";

  private final Type groupType;

  private final Type hostedType;

  private final Format maven2Format;

  @Autowired
  public PurgeMavenUnusedSnapshotsTask(
      @Qualifier(GroupType.NAME) final Type groupType,
      @Qualifier(HostedType.NAME) final Type hostedType,
      @Qualifier(Maven2Format.NAME) final Format maven2Format)
  {
    this.groupType = checkNotNull(groupType);
    this.hostedType = checkNotNull(hostedType);
    this.maven2Format = checkNotNull(maven2Format);
  }

  @Override
  protected void execute(final Repository repository) {
    repository.facet(PurgeUnusedSnapshotsFacet.class)
        .purgeUnusedSnapshots(getConfiguration().getInteger(LAST_USED_FIELD_ID, -1));
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return hasExpectedFormat(repository) && !isReleaseRepo(repository);
  }

  /**
   * Validates if the passed repository has the expected format to run the task
   *
   * @param repository the repository to be validated
   * @return a {@link Boolean} flag representing if the format is valid or not
   */
  private boolean hasExpectedFormat(final Repository repository) {
    return maven2Format.equals(repository.getFormat())
        && (hostedType.equals(repository.getType()) || groupType.equals(repository.getType()));
  }

  private boolean isReleaseRepo(final Repository repository) {
    VersionPolicy versionPolicy = repository.facet(MavenFacet.class).getVersionPolicy();
    return VersionPolicy.RELEASE.equals(versionPolicy);
  }

  @Override
  public String getMessage() {
    return "Purge unused Maven snapshot versions from " + getRepositoryField();
  }
}
