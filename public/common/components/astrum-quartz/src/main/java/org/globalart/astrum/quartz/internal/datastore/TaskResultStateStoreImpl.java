
package org.globalart.astrum.quartz.internal.datastore;

import java.util.Collections;
import java.util.Date;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.quartz.internal.task.QuartzTaskInfo;
import org.globalart.astrum.quartz.internal.task.QuartzTaskUtils;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskState;
import org.globalart.astrum.scheduling.spi.TaskResultState;
import org.globalart.astrum.scheduling.spi.TaskResultStateStore;
import org.globalart.astrum.transaction.Transactional;

import org.quartz.JobDataMap;
import org.quartz.JobKey;
import org.quartz.Scheduler;
import org.quartz.SchedulerException;
import org.quartz.Trigger;
import org.springframework.context.annotation.Lazy;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.quartz.TriggerKey.triggerKey;
import static org.globalart.astrum.quartz.internal.task.QuartzTaskUtils.updateJobData;
import org.springframework.stereotype.Component;

@Component
public class TaskResultStateStoreImpl
    extends ConfigStoreSupport<QuartzDAO>
    implements TaskResultStateStore
{
  protected Scheduler scheduler;

  @Autowired
  public TaskResultStateStoreImpl(final DataSessionSupplier sessionSupplier, @Lazy final Scheduler scheduler) {
    super(sessionSupplier);
    this.scheduler = checkNotNull(scheduler);
  }

  @Override
  public Optional<TaskResultState> getState(final TaskInfo taskInfo) {
    return Optional.ofNullable(taskInfo)
        .map(QuartzTaskInfo.class::cast)
        .map(QuartzTaskInfo::getJobKey)
        .map(JobKey::getName)
        .flatMap(this::getQuartzState)
        .map(Collections::singletonList)
        .map(jobStates -> aggregate(taskInfo, jobStates));
  }

  @Override
  public void updateJobDataMap(final TaskInfo taskInfo) {
    String jobName = ((QuartzTaskInfo) taskInfo).getJobKey().getName();
    JobDataMap jobDataMap = new JobDataMap();
    updateJobData(jobDataMap, taskInfo.getConfiguration());

    // Don't persist for any tasks that may run when frozen, as exceptions may occur
    if (!taskInfo.getConfiguration().getBoolean(TaskConfiguration.RUN_WHEN_FROZEN, false)) {
      doUpdateJobDataMap(jobName, jobDataMap);
    }
  }

  @Transactional
  protected List<QuartzTaskStateData> getQuartzStates() {
    return dao().getStates();
  }

  @Transactional
  protected Optional<QuartzTaskStateData> getQuartzState(final String taskId) {
    return dao().getState(taskId);
  }

  @Transactional
  protected void doUpdateJobDataMap(final String jobName, final JobDataMap jobDataMap) {
    QuartzTaskStateData quartzTaskStateData = new QuartzTaskStateData();
    quartzTaskStateData.setJobName(jobName);
    quartzTaskStateData.setJobData(jobDataMap);
    dao().updateJobDataMap(quartzTaskStateData);
  }

  private TaskResultState aggregate(final TaskInfo taskInfo, final List<QuartzTaskStateData> jobStates) {
    TaskConfiguration taskConfiguration = jobStates.stream()
        .map(QuartzTaskStateData::getJobData)
        .filter(Objects::nonNull)
        .map(QuartzTaskUtils::configurationOf)
        .filter(Objects::nonNull)
        .findAny()
        .orElse(null);

    if (taskConfiguration == null) {
      return null;
    }

    Date nextFireTime = null;
    JobKey jobKey = ((QuartzTaskInfo) taskInfo).getJobKey();
    try {
      nextFireTime = Optional.ofNullable(scheduler.getTrigger(triggerKey(jobKey.getName(), jobKey.getGroup())))
          .map(Trigger::getNextFireTime)
          .orElseGet(taskInfo.getCurrentState()::getNextRun);
    }
    catch (SchedulerException e) {
      log.debug("An error occurred finding the next fire time for {}", taskConfiguration.getId(), e);
      nextFireTime = taskInfo.getCurrentState().getNextRun();
    }

    boolean running = jobStates.stream()
        .map(QuartzTaskStateData::getState)
        .anyMatch("EXECUTING"::equals);

    return new TaskResultState(taskConfiguration.getId(), running ? TaskState.RUNNING : TaskState.WAITING,
        nextFireTime, taskConfiguration.getLastRunState(), taskConfiguration.getProgress());
  }
}
