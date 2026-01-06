
package org.globalart.astrum.quartz.internal.bulkread;

import java.sql.ResultSet;
import java.sql.SQLException;

import org.quartz.TriggerKey;

import static org.quartz.TriggerKey.triggerKey;

/**
 * Immutable data holder for simple trigger data from QRTZ_SIMPLE_TRIGGERS table.
 */
record SimpleTriggerData(
    String triggerName,
    String triggerGroup,
    int repeatCount,
    long repeatInterval,
    int timesTriggered)
{
  static SimpleTriggerData fromResultSet(final ResultSet rs) throws SQLException {
    return new SimpleTriggerData(
        rs.getString("TRIGGER_NAME"),
        rs.getString("TRIGGER_GROUP"),
        rs.getInt("REPEAT_COUNT"),
        rs.getLong("REPEAT_INTERVAL"),
        rs.getInt("TIMES_TRIGGERED"));
  }

  /**
   * Returns the TriggerKey for this simple trigger.
   */
  TriggerKey getKey() {
    return triggerKey(triggerName, triggerGroup);
  }
}
