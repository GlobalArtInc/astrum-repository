
package org.globalart.astrum.scheduling.schedule;

import java.io.IOException;

import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.databind.SerializerProvider;
import com.fasterxml.jackson.databind.ser.std.StdSerializer;

import static org.globalart.astrum.scheduling.schedule.ScheduleConstants.CLASS_TYPE;
import static org.globalart.astrum.scheduling.schedule.ScheduleConstants.CRON_EXPRESSION;
import static org.globalart.astrum.scheduling.schedule.ScheduleConstants.DAYS_TO_RUN;
import static org.globalart.astrum.scheduling.schedule.ScheduleConstants.START_AT;
import static org.globalart.astrum.scheduling.schedule.ScheduleConstants.TIME_ZONE;

/**
 * Serializer for the {@link Schedule} class.
 *
 * @since 3.30
 */
public class ScheduleSerializer
    extends StdSerializer<Schedule>
{
  private static final long serialVersionUID = 6929523949071161349L;

  public ScheduleSerializer() {
    super(Schedule.class);
  }

  @Override
  public void serialize(
      final Schedule value,
      final JsonGenerator jgen,
      final SerializerProvider provider) throws IOException
  {
    jgen.writeStartObject();
    jgen.writeStringField(CLASS_TYPE, value.getType());
    if (value instanceof Monthly) {
      Monthly monthly = (Monthly) value;
      jgen.writeObjectField(START_AT, monthly.getStartAt());
      jgen.writeObjectField(DAYS_TO_RUN, monthly.getDaysToRun());
    }
    else if (value instanceof Weekly) {
      Weekly daily = (Weekly) value;
      jgen.writeObjectField(START_AT, daily.getStartAt());
      jgen.writeObjectField(DAYS_TO_RUN, daily.getDaysToRun());
    }
    else if (value instanceof Daily) {
      Daily daily = (Daily) value;
      jgen.writeObjectField(START_AT, daily.getStartAt());
    }
    else if (value instanceof Hourly) {
      Hourly hourly = (Hourly) value;
      jgen.writeObjectField(START_AT, hourly.getStartAt());
    }
    else if (value instanceof Once) {
      Once once = (Once) value;
      jgen.writeObjectField(START_AT, once.getStartAt());
    }
    else if (value instanceof Cron) {
      Cron cron = (Cron) value;
      jgen.writeObjectField(START_AT, cron.getStartAt());
      jgen.writeObjectField(CRON_EXPRESSION, cron.getCronExpression());
      jgen.writeObjectField(TIME_ZONE, cron.getTimeZone());
    }
    jgen.writeEndObject();
  }
}
