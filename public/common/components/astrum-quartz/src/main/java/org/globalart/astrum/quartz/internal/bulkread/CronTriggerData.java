
package org.globalart.astrum.quartz.internal.bulkread;

import java.sql.ResultSet;
import java.sql.SQLException;

import org.quartz.TriggerKey;

import static org.quartz.TriggerKey.triggerKey;

/**
 * Immutable data holder for cron trigger data from QRTZ_CRON_TRIGGERS table.
 */
record CronTriggerData(String triggerName, String triggerGroup, String cronExpression, String timeZoneId)
{
  static CronTriggerData fromResultSet(final ResultSet rs) throws SQLException {
    return new CronTriggerData(
        rs.getString("TRIGGER_NAME"),
        rs.getString("TRIGGER_GROUP"),
        rs.getString("CRON_EXPRESSION"),
        rs.getString("TIME_ZONE_ID"));
  }

  /**
   * Returns the TriggerKey for this cron trigger.
   */
  TriggerKey getKey() {
    return triggerKey(triggerName, triggerGroup);
  }
}
