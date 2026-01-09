
package org.globalart.astrum.scheduling.constraints;

import java.util.Date;

import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;

import jakarta.validation.ConstraintValidatorContext;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CronExpressionValidatorTest
{
  private CronExpressionValidator validator;

  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private ScheduleFactory scheduleFactory;

  @Mock
  private ConstraintValidatorContext context;

  @Before
  public void setup() {
    when(taskScheduler.getScheduleFactory()).thenReturn(scheduleFactory);

    validator = new CronExpressionValidator(taskScheduler);
  }

  @Test
  public void isValidStripsJavaElFromExceptionMessage() {
    String value = "value ${java el} {el}";
    when(scheduleFactory.cron(any(Date.class), eq(value))).thenThrow(new IllegalArgumentException(value));
    when(context.buildConstraintViolationWithTemplate(any()))
        .thenReturn(mock(ConstraintValidatorContext.ConstraintViolationBuilder.class));

    boolean isValid = validator.isValid(value, context);

    assertThat(isValid, is(false));
    verify(context).buildConstraintViolationWithTemplate("value {java el} {el}");
  }
}
