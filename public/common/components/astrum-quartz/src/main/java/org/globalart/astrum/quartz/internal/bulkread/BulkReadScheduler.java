
package org.globalart.astrum.quartz.internal.bulkread;

import java.util.Map;

import org.quartz.JobDetail;
import org.quartz.JobKey;
import org.quartz.Scheduler;
import org.quartz.SchedulerException;
import org.quartz.Trigger;
import org.quartz.TriggerKey;

/**
 * Extended Scheduler interface with optimized bulk read methods.
 * This avoids the N+1 query problem when fetching multiple jobs with triggers.
 */
public interface BulkReadScheduler
    extends Scheduler
{
  /**
   * Retrieves all job details for a specific group using a single optimized query.
   *
   * @param groupName the job group name
   * @return map of job keys to their associated job details
   * @throws SchedulerException if there is an error retrieving the job details
   */
  Map<JobKey, JobDetail> getJobDetailsForGroup(String groupName) throws SchedulerException;

  /**
   * Retrieves all triggers for a specific group using optimized bulk queries.
   * This method executes 3 queries (QRTZ_TRIGGERS, QRTZ_CRON_TRIGGERS, QRTZ_SIMPLE_TRIGGERS).
   *
   * @param groupName the trigger group name
   * @return map of trigger keys to their associated triggers
   * @throws SchedulerException if there is an error retrieving the triggers
   */
  Map<TriggerKey, Trigger> getTriggersForGroup(String groupName) throws SchedulerException;

  /**
   * Returns the underlying delegate Scheduler.
   *
   * @return the delegate Scheduler
   */
  Scheduler getDelegate();
}
