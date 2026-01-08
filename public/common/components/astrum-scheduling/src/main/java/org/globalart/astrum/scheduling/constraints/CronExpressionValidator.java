
package org.globalart.astrum.scheduling.constraints;

import java.util.Date;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import org.springframework.context.annotation.Lazy;
import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link CronExpression} validator.
 *
 * @see ScheduleFactory#cron(Date, String)
 * @since 3.0
 */
@Lazy
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CronExpressionValidator
    extends ConstraintValidatorSupport<CronExpression, String>
{
  private final ScheduleFactory scheduleFactory;

  @Autowired
  public CronExpressionValidator(final TaskScheduler taskScheduler) {
    checkNotNull(taskScheduler);
    this.scheduleFactory = taskScheduler.getScheduleFactory();
  }

  @Override
  public void initialize(final CronExpression annotation) {
    // nop
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    try {
      scheduleFactory.cron(new Date(), value);
    }
    catch (IllegalArgumentException e) {
      context.disableDefaultConstraintViolation();
      context.buildConstraintViolationWithTemplate(getEscapeHelper().stripJavaEl(e.getMessage()))
          .addConstraintViolation();
      return false;
    }
    return true;
  }
}
