
package org.globalart.astrum.quartz.internal;

import javax.annotation.Nullable;

import org.quartz.Job;
import org.quartz.Scheduler;
import org.quartz.SchedulerException;
import org.quartz.spi.JobFactory;
import org.quartz.spi.TriggerFiredBundle;
import org.springframework.beans.BeansException;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.context.ApplicationContext;
import org.springframework.context.ApplicationContextAware;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Spring-aware {@link JobFactory}.
 *
 * @since 3.0
 */
@Component
public class JobFactoryImpl
    implements JobFactory, ApplicationContextAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private ApplicationContext applicationContext;

  @Override
  public Job newJob(final TriggerFiredBundle bundle, final Scheduler scheduler) throws SchedulerException {
    Class<? extends Job> type = bundle.getJobDetail().getJobClass();
    log.debug("New job: {}", type);

    ObjectProvider<? extends Job> beanEntry = locate(type);
    if (beanEntry == null) {
      throw new SchedulerException("Missing Job component for type: " + type.getName());
    }

    return beanEntry.getIfUnique(); // to support non-singletons
  }

  @Nullable
  private ObjectProvider<? extends Job> locate(final Class<? extends Job> jobType) {
    return applicationContext.getBeanProvider(jobType);
  }

  @Override
  public void setApplicationContext(final ApplicationContext applicationContext) throws BeansException {
    this.applicationContext = applicationContext;
  }
}
