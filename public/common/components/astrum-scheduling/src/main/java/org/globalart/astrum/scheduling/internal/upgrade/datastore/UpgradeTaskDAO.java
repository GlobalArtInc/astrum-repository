
package org.globalart.astrum.scheduling.internal.upgrade.datastore;

import java.time.OffsetDateTime;
import java.util.Optional;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * Data access for a queue of tasks triggered by migration steps which Astrum needs to complete.
 */
public interface UpgradeTaskDAO
    extends DataAccess
{
  int deleteByTaskId(String taskId);

  /**
   * Browse existing entities.
   */
  Iterable<UpgradeTaskData> browse();

  /**
   * Create a new entity.
   */
  int create(UpgradeTaskData entity);

  /**
   * Retrieve the entity with the given id.
   */
  Optional<UpgradeTaskData> read(int id);

  /**
   * Update an existing entity.
   *
   */
  boolean update(UpgradeTaskData entity);

  /**
   * Updates the status of an upgrade task
   *
   * @param taskId the id of the task
   * @param status the status message
   * @return the number of rows modified
   */
  int setStatus(@Param("taskId") String taskId, @Param("status") String status);

  /**
   * Find the next upgrade task as ordered
   */
  Optional<UpgradeTaskData> next();

  Iterable<UpgradeTaskData> browseBefore(@Param("before") OffsetDateTime before);
}
