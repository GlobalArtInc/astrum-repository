
package org.globalart.astrum.quartz.internal.bulkread;

import java.sql.ResultSet;
import java.sql.SQLException;

import org.quartz.JobKey;

import static org.quartz.JobKey.jobKey;

/**
 * Holds data from QRTZ_JOB_DETAILS table.
 * Note: IS_NONCONCURRENT and IS_UPDATE_DATA are not included as they are derived
 * from Job class annotations ({@link org.quartz.DisallowConcurrentExecution @DisallowConcurrentExecution} and
 * {@link org.quartz.PersistJobDataAfterExecution @PersistJobDataAfterExecution}).
 */
record JobDetailData(
    String schedName,
    String jobName,
    String jobGroup,
    String description,
    String jobClassName,
    boolean isDurable,
    boolean requestsRecovery,
    byte[] jobData)
{
  /**
   * Creates a JobDetailData from a ResultSet row.
   */
  static JobDetailData fromResultSet(final ResultSet rs) throws SQLException {
    return new JobDetailData(
        rs.getString("SCHED_NAME"),
        rs.getString("JOB_NAME"),
        rs.getString("JOB_GROUP"),
        rs.getString("DESCRIPTION"),
        rs.getString("JOB_CLASS_NAME"),
        rs.getBoolean("IS_DURABLE"),
        rs.getBoolean("REQUESTS_RECOVERY"),
        rs.getBytes("JOB_DATA"));
  }

  /**
   * Returns the JobKey for this job detail.
   */
  JobKey getKey() {
    return jobKey(jobName, jobGroup);
  }
}
