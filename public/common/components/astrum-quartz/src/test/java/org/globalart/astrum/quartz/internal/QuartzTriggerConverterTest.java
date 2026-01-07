
package org.globalart.astrum.quartz.internal;

import java.util.Calendar;
import java.util.Date;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.scheduling.schedule.Hourly;
import org.globalart.astrum.scheduling.schedule.Monthly;
import org.globalart.astrum.scheduling.schedule.Monthly.CalendarDay;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;
import org.globalart.astrum.scheduling.schedule.Weekly;
import org.globalart.astrum.scheduling.schedule.Weekly.Weekday;

import org.junit.Before;
import org.junit.Test;
import org.mockito.internal.util.collections.Sets;
import org.quartz.CronTrigger;
import org.quartz.Trigger;
import org.quartz.TriggerBuilder;
import org.quartz.impl.triggers.SimpleTriggerImpl;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.instanceOf;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.quartz.internal.QuartzTriggerConverter.cronTimeParts;

/**
 * Tests for {@link QuartzTriggerConverter}
 */
public class QuartzTriggerConverterTest
{
  private QuartzTriggerConverter underTest;

  private ScheduleFactory scheduleFactory;

  @Before
  public void setUp() throws Exception {
    this.scheduleFactory = new QuartzScheduleFactory();
    this.underTest = new QuartzTriggerConverter(scheduleFactory);
  }

  @Test
  public void verifyCronTimeParts() {
    Calendar cal = Calendar.getInstance();
    cal.set(Calendar.HOUR_OF_DAY, 7);
    cal.set(Calendar.MINUTE, 15);
    assertThat(cronTimeParts(cal.getTime()), equalTo("0 15 7"));

    cal.set(Calendar.HOUR_OF_DAY, 22);
    cal.set(Calendar.MINUTE, 59);
    assertThat(cronTimeParts(cal.getTime()), equalTo("0 59 22"));

    cal.set(Calendar.HOUR_OF_DAY, 0);
    cal.set(Calendar.MINUTE, 0);
    assertThat(cronTimeParts(cal.getTime()), equalTo("0 0 0"));
  }

  @Test
  public void hourly() {
    Date now = new Date();
    Hourly hourly = scheduleFactory.hourly(now);
    TriggerBuilder triggerBuilder = underTest.convert(hourly);
    Trigger trigger = triggerBuilder.build();

    assertThat(trigger, instanceOf(SimpleTriggerImpl.class));
    assertThat(trigger.getFireTimeAfter(new Date(now.getTime())),
        equalTo(new Date(now.getTime() + TimeUnit.HOURS.toMillis(1L))));
  }

  @Test
  public void weekly1() {
    Weekly weekly = scheduleFactory.weekly(new Date(), Sets.newSet(Weekday.SAT));
    TriggerBuilder triggerBuilder = underTest.convert(weekly);
    Trigger trigger = triggerBuilder.build();

    assertThat(trigger, instanceOf(CronTrigger.class));
    String cronExpression = ((CronTrigger) trigger).getCronExpression();
    assertThat(cronExpression, equalTo(cronTimeParts(weekly.getStartAt()) + " ? * SAT"));
  }

  @Test
  public void weekly2() {
    Weekly weekly = scheduleFactory.weekly(new Date(), Sets.newSet(Weekday.SAT, Weekday.FRI));
    TriggerBuilder triggerBuilder = underTest.convert(weekly);
    Trigger trigger = triggerBuilder.build();

    assertThat(trigger, instanceOf(CronTrigger.class));
    String cronExpression = ((CronTrigger) trigger).getCronExpression();
    assertThat(cronExpression, equalTo(cronTimeParts(weekly.getStartAt()) + " ? * FRI,SAT"));
  }

  @Test
  public void monthly1() {
    Monthly monthly = scheduleFactory.monthly(new Date(), CalendarDay.days(2));
    TriggerBuilder triggerBuilder = underTest.convert(monthly);
    Trigger trigger = triggerBuilder.build();

    assertThat(trigger, instanceOf(CronTrigger.class));
    String cronExpression = ((CronTrigger) trigger).getCronExpression();
    assertThat(cronExpression, equalTo(cronTimeParts(monthly.getStartAt()) + " 2 * ?"));
  }

  @Test
  public void monthly2() {
    Monthly monthly = scheduleFactory.monthly(new Date(), CalendarDay.days(1, 2, 3, 10, 11, 12));
    TriggerBuilder triggerBuilder = underTest.convert(monthly);
    Trigger trigger = triggerBuilder.build();

    assertThat(trigger, instanceOf(CronTrigger.class));
    String cronExpression = ((CronTrigger) trigger).getCronExpression();
    assertThat(cronExpression, equalTo(
        cronTimeParts(monthly.getStartAt()) + " 1,2,3,10,11,12 * ?"));
  }

  @Test
  public void monthly3() {
    Monthly monthly = scheduleFactory.monthly(new Date(), Sets.newSet(CalendarDay.lastDay()));
    TriggerBuilder triggerBuilder = underTest.convert(monthly);
    Trigger trigger = triggerBuilder.build();

    assertThat(trigger, instanceOf(CronTrigger.class));
    String cronExpression = ((CronTrigger) trigger).getCronExpression();
    assertThat(cronExpression, equalTo(cronTimeParts(monthly.getStartAt()) + " L * ?"));
  }
}
