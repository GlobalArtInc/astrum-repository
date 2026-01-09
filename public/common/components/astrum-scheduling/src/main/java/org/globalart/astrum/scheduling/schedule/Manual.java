
package org.globalart.astrum.scheduling.schedule;

/**
 * Schedule that never executes, is meant for manual execution (from code or via UI by user).
 *
 * @see ScheduleFactory#manual()
 */
public class Manual
    extends Schedule
{
  public static final String TYPE = "manual";

  public Manual() {
    super(TYPE);
  }
}
