
package org.globalart.astrum.quartz.internal.datastore;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.datastore.api.DataAccess;

import org.quartz.impl.jdbcjobstore.JobStoreTX;

/**
 * Quartz data access object used to create the schema that {@link JobStoreTX} expects.
 *
 * @since 3.19
 */
public interface QuartzDAO
    extends DataAccess
{
  List<QuartzTaskStateData> getStates();

  Optional<QuartzTaskStateData> getState(String jobName);

  void updateJobDataMap(QuartzTaskStateData taskState);
}
