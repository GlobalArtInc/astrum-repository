
package org.globalart.astrum.cleanup.storage;

import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.common.lifecycle.Lifecycle;

/**
 * @since 3.14
 */
public interface CleanupPolicyStorage
    extends Lifecycle
{
  /**
   * @since 3.20
   * @return the count of cleanup policies stored
   */
  long count();

  /**
   * Adds a cleanup policy
   *
   * @param item to be added
   * @return the added {@link CleanupPolicy}
   */
  CleanupPolicy add(CleanupPolicy item);

  /**
   * Updates stored cleanup policy if it exists.
   *
   * @param cleanupPolicy to be updated
   * @return the updated {@link CleanupPolicy}
   */
  CleanupPolicy update(CleanupPolicy cleanupPolicy);

  /**
   * Deletes stored cleanup policy if it exists.
   *
   * @param cleanupPolicy cleanup to be deleted
   */
  void remove(final CleanupPolicy cleanupPolicy);

  /**
   * Retrieves a single stored policy
   * 
   * @param cleanupPolicyName name of cleanup policy to retrieve
   * @return {@link CleanupPolicy}
   */
  @Nullable
  CleanupPolicy get(final String cleanupPolicyName);

  /**
   * Retrieves stored cleanup policies
   *
   * @return list of cleanup policies (never null)
   */
  List<CleanupPolicy> getAll();

  /**
   * Retrieves stored clean policies related to a {@link org.globalart.astrum.repository.Format} type
   * 
   * @return list of cleanup policies
   */
  List<CleanupPolicy> getAllByFormat(final String format);

  /**
   * Checks whether a cleanup policy with a given name exists.
   *
   * @param cleanupPolicyName name of cleanup policy to check for existence.
   * @return true if it existed by the given name ignoring case, false otherwise.
   */
  boolean exists(final String cleanupPolicyName);

  /**
   * Create a new instance of a {@link CleanupPolicy} suitable for the backing implementation.
   *
   * @since 3.20
   */
  CleanupPolicy newCleanupPolicy();
}
