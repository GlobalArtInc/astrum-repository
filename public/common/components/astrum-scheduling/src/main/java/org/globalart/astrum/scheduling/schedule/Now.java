
package org.globalart.astrum.scheduling.schedule;

/**
 * Schedule that executes once at the moment task is being scheduled with.
 *
 * @see ScheduleFactory#now()
 */
public class Now
    extends Schedule
{
  public static final String TYPE = "now";

  public Now() {
    super(TYPE);
  }
}
