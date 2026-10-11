package org.globalart.astrum.scheduling.api;

import java.util.Arrays;

import io.swagger.v3.oas.annotations.media.Schema;

/**
 * When a task created or updated through the REST API runs.
 *
 * @since 3.96
 */
@Schema(description = "The schedule of a task")
public class FrequencyXO
{
  @Schema(description = "The type of schedule", example = "daily",
      allowableValues = {"manual", "once", "hourly", "daily", "weekly", "monthly", "cron"})
  private String schedule;

  @Schema(description = "The first run of the task, in milliseconds since the epoch; required for every schedule "
      + "but manual and cron", example = "1735693200000")
  private Long startDate;

  @Schema(description = "The offset of the client's time zone, used to interpret the start date, the days and a cron "
      + "expression", example = "+02:00")
  private String timeZoneOffset;

  @Schema(description = "The days a weekly (1=Sunday to 7=Saturday) or monthly (1 to 31, 999=last day of the month) "
      + "task runs on", example = "[2, 4]")
  private Integer[] recurringDays;

  @Schema(description = "The Quartz cron expression of a cron schedule", example = "0 0 1 * * ?")
  private String cronExpression;

  public String getSchedule() {
    return schedule;
  }

  public void setSchedule(final String schedule) {
    this.schedule = schedule;
  }

  public Long getStartDate() {
    return startDate;
  }

  public void setStartDate(final Long startDate) {
    this.startDate = startDate;
  }

  public String getTimeZoneOffset() {
    return timeZoneOffset;
  }

  public void setTimeZoneOffset(final String timeZoneOffset) {
    this.timeZoneOffset = timeZoneOffset;
  }

  public Integer[] getRecurringDays() {
    return recurringDays;
  }

  public void setRecurringDays(final Integer[] recurringDays) {
    this.recurringDays = recurringDays;
  }

  public String getCronExpression() {
    return cronExpression;
  }

  public void setCronExpression(final String cronExpression) {
    this.cronExpression = cronExpression;
  }

  @Override
  public String toString() {
    return "FrequencyXO{" +
        "schedule='" + schedule + '\'' +
        ", startDate=" + startDate +
        ", timeZoneOffset='" + timeZoneOffset + '\'' +
        ", recurringDays=" + Arrays.toString(recurringDays) +
        ", cronExpression='" + cronExpression + '\'' +
        '}';
  }
}
