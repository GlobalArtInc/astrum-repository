
package org.globalart.astrum.content.maven.store;

import java.time.LocalDate;
import java.util.Collection;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE;

/**
 * @since 3.29
 */
public class Maven2ComponentStore
    extends ComponentStore<Maven2ComponentDAO>
{
  @Autowired
  public Maven2ComponentStore(
      final DataSessionSupplier sessionSupplier,
      @Value(DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean clustered,
      final String storeName)
  {
    super(sessionSupplier, clustered, storeName, Maven2ComponentDAO.class);
  }

  /**
   * Updates the maven base_version of the given component in the content data store.
   *
   * @param component the component to update
   */
  @Transactional
  public void updateBaseVersion(final Maven2ComponentData component) {
    dao().updateBaseVersion(component);
  }

  @Transactional
  public Set<GAV> findGavsWithSnaphots(final int repositoryId, final int minimumRetained) {
    return dao().findGavsWithSnaphots(repositoryId, minimumRetained);
  }

  @Transactional
  public List<Maven2ComponentData> findComponentsForGav(
      final int repositoryId,
      final String name,
      final String group,
      final String baseVersion,
      final String releaseVersion)
  {
    return dao().findComponentsForGav(repositoryId, name, group, baseVersion, releaseVersion);
  }

  @Transactional
  public Set<String> getBaseVersions(final int repositoryId, final String namespace, final String name) {
    return dao().getBaseVersions(repositoryId, namespace, name);
  }

  @Transactional
  public int[] selectSnapshotsAfterRelease(final int repositoryId, final int gracePeriod) {
    return dao().selectSnapshotsAfterRelease(gracePeriod, repositoryId);
  }

  /**
   * Selects snapshot components ids last used before provided date
   *
   * @param repositoryId the repository to select from
   * @param olderThan selects component before this date
   * @param limit limit the selection
   * @return snapshot components last used before provided date
   */
  @Transactional
  public Collection<Integer> selectUnusedSnapshots(
      final int repositoryId,
      final LocalDate olderThan,
      final long limit)
  {
    return dao().selectUnusedSnapshots(repositoryId, olderThan, limit);
  }
}
