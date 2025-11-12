
package org.globalart.astrum.internal.log.overrides.datastore;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * {@link LoggingOverridesData} DataAccess
 */
public interface LoggingOverridesDAO
    extends DataAccess
{
  /**
   * Create new record
   *
   * @param data represents logger configuration
   */
  void createRecord(LoggingOverridesData data);

  /**
   * Return all records stored in DB, the continuationToken to be used when amount more than single page (>1000 rows)
   *
   * @param continuationToken the record id used for pagination
   * @return all records
   */
  Continuation<LoggingOverridesData> readRecords(@Nullable @Param("continuationToken") String continuationToken);

  /**
   * Delete single record by provided 'NAME'
   *
   * @param name the record ID
   */
  void deleteRecord(@Param("name") String name);

  /**
   * Delete all records
   */
  void deleteAllRecords();

  /**
   * Update record by name
   *
   * @param data represents logger configuration
   */
  void updateRecord(LoggingOverridesData data);
}
