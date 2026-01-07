
package org.globalart.astrum.quartz.internal;

import java.util.Date;

import org.junit.Before;
import org.junit.Test;

public class QuartzScheduleFactoryTest
{
  private QuartzScheduleFactory underTest;

  @Before
  public void setUp() {
    underTest = new QuartzScheduleFactory();
  }

  /**
   * Parses cron expression: "seconds minutes hours day-of-month month day-of-week year"
   */
  private void cron(String pattern) {
    System.out.println("Parsing: " + pattern);
    underTest.cron(new Date(), pattern);
  }

  @Test(expected = IllegalArgumentException.class)
  public void testBadOneMonthInsteadOfDay() {
    cron("0 0 0 ? * JAN");
  }

  @Test(expected = IllegalArgumentException.class)
  public void testBadOneDotInDays() {
    cron("0 0 0 ? * FRI.,SAT");
  }

  @Test(expected = IllegalArgumentException.class)
  public void testBadOneQuestionmarkAmongDays() {
    cron("1 2 4 ?,1,2,3,10,11,12 * ?");
  }

  @Test(expected = IllegalArgumentException.class)
  public void testBadOneLAtBadPosition() {
    cron("0 0 L L * ?");
  }
}
