
package org.globalart.astrum.quartz.internal.bulkread;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.ObjectInputStream;
import java.sql.SQLException;
import java.util.Date;
import java.util.Map;
import java.util.TimeZone;

import org.quartz.Job;
import org.quartz.JobDataMap;
import org.quartz.JobDetail;
import org.quartz.impl.JobDetailImpl;
import org.quartz.impl.triggers.CronTriggerImpl;
import org.quartz.impl.triggers.SimpleTriggerImpl;
import org.quartz.spi.OperableTrigger;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.quartz.JobKey.jobKey;
import static org.quartz.TriggerKey.triggerKey;

/**
 * Helper class to build Quartz objects from database ResultSets. Handles conversion of raw DB data to JobDetail and
 * Trigger objects.
 */
class QuartzObjectBuilder
{
  private static final Logger log = LoggerFactory.getLogger(QuartzObjectBuilder.class);

  /**
   * Builds a JobDetail from JobDetailData.
   */
  static JobDetail buildJobDetail(final JobDetailData data) throws SQLException {
    try {
      JobDetailImpl job = new JobDetailImpl();

      job.setName(data.jobName());
      job.setGroup(data.jobGroup());
      job.setDescription(data.description());
      job.setDurability(data.isDurable());
      job.setRequestsRecovery(data.requestsRecovery());

      // Load job class
      Class<? extends Job> jobClass = (Class<? extends Job>) Class.forName(data.jobClassName());
      job.setJobClass(jobClass);

      // Deserialize job data map
      if (data.jobData() != null && data.jobData().length > 0) {
        JobDataMap jobDataMap = deserializeJobData(data.jobData());
        job.setJobDataMap(jobDataMap);
      }

      return job;
    }
    catch (ClassNotFoundException | IOException e) {
      throw new SQLException("Failed to build job detail from data", e);
    }
  }

  /**
   * Deserializes a JobDataMap from bytes.
   */
  private static JobDataMap deserializeJobData(final byte[] data) throws IOException, ClassNotFoundException {
    try (ByteArrayInputStream bis = new ByteArrayInputStream(data);
        ObjectInputStream ois = new ObjectInputStream(bis)) {
      Object jobData = ois.readObject();
      if (jobData instanceof JobDataMap jobDataMap) {
        return jobDataMap;
      }
      else if (jobData instanceof Map<?, ?> map) {
        return new JobDataMap(map);
      }
      else {
        log.error("Unexpected job data type: {}", jobData.getClass().getName());
        throw new IOException("Unexpected job data type: " + jobData.getClass().getName());
      }
    }
  }

  /**
   * Builds a cron trigger from TriggerData and CronTriggerData.
   */
  static OperableTrigger buildCronTrigger(
      final TriggerData triggerData,
      final CronTriggerData cronData) throws SQLException
  {
    CronTriggerImpl trigger = new CronTriggerImpl();
    populateBaseTrigger(trigger, triggerData);

    if (cronData != null) {
      try {
        trigger.setCronExpression(cronData.cronExpression());
        if (cronData.timeZoneId() != null) {
          trigger.setTimeZone(TimeZone.getTimeZone(cronData.timeZoneId()));
        }
      }
      catch (Exception e) {
        throw new SQLException("Failed to set cron expression", e);
      }
    }

    return trigger;
  }

  /**
   * Builds a simple trigger from TriggerData and SimpleTriggerData.
   */
  static OperableTrigger buildSimpleTrigger(
      final TriggerData triggerData,
      final SimpleTriggerData simpleData) throws SQLException
  {
    SimpleTriggerImpl trigger = new SimpleTriggerImpl();
    populateBaseTrigger(trigger, triggerData);

    if (simpleData != null) {
      trigger.setRepeatCount(simpleData.repeatCount());
      trigger.setRepeatInterval(simpleData.repeatInterval());
      trigger.setTimesTriggered(simpleData.timesTriggered());
    }

    return trigger;
  }

  /**
   * Builds a basic trigger (for unknown types) from TriggerData.
   */
  static OperableTrigger buildBasicTrigger(final TriggerData triggerData) throws SQLException {
    SimpleTriggerImpl trigger = new SimpleTriggerImpl();
    populateBaseTrigger(trigger, triggerData);
    return trigger;
  }

  /**
   * Populates base trigger fields from TriggerData object.
   */
  private static void populateBaseTrigger(final OperableTrigger trigger, final TriggerData data) throws SQLException {
    trigger.setKey(triggerKey(data.triggerName(), data.triggerGroup()));
    trigger.setJobKey(jobKey(data.jobName(), data.jobGroup()));
    trigger.setDescription(data.description());

    if (data.nextFireTime() > 0) {
      trigger.setNextFireTime(new Date(data.nextFireTime()));
    }

    if (data.prevFireTime() > 0) {
      trigger.setPreviousFireTime(new Date(data.prevFireTime()));
    }

    if (data.startTime() > 0) {
      trigger.setStartTime(new Date(data.startTime()));
    }

    if (data.endTime() > 0) {
      trigger.setEndTime(new Date(data.endTime()));
    }

    trigger.setPriority(data.priority());
    trigger.setCalendarName(data.calendarName());
    trigger.setMisfireInstruction(data.misfireInstr());

    // Deserialize trigger job data map
    if (data.jobData() != null && data.jobData().length > 0) {
      try {
        JobDataMap jobDataMap = deserializeJobData(data.jobData());
        trigger.setJobDataMap(jobDataMap);
      }
      catch (IOException | ClassNotFoundException e) {
        throw new SQLException("Failed to deserialize trigger job data", e);
      }
    }
  }
}
