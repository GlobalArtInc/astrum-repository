
package org.globalart.astrum.quartz.internal.datastore;

import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.function.Function;
import java.util.stream.Collectors;
import javax.annotation.Nullable;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.log.LastShutdownTimeService;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.thread.TcclBlock;
import org.globalart.astrum.quartz.internal.QuartzSchedulerSPI;
import org.globalart.astrum.quartz.internal.bulkread.BulkReadScheduler;
import org.globalart.astrum.quartz.internal.task.QuartzTaskInfo;
import org.globalart.astrum.quartz.internal.task.QuartzTaskJobListener;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.scheduling.spi.SchedulerSPI;
import org.globalart.astrum.thread.DatabaseStatusDelayedExecutor;

import com.google.common.eventbus.Subscribe;
import jakarta.inject.Provider;
import org.quartz.JobDetail;
import org.quartz.JobKey;
import org.quartz.JobListener;
import org.quartz.SchedulerException;
import org.quartz.Trigger;
import org.quartz.TriggerKey;
import org.quartz.listeners.SchedulerListenerSupport;
import org.quartz.spi.JobStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.quartz.TriggerKey.triggerKey;
import static org.quartz.impl.matchers.GroupMatcher.jobGroupEquals;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;
import static org.globalart.astrum.quartz.internal.task.QuartzTaskJobListener.listenerName;

/**
 * Quartz {@link SchedulerSPI}.
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class DatastoreQuartzSchedulerSPI
    extends QuartzSchedulerSPI
    implements EventAware
{
  private final Object mutex = new Object();

  @Autowired
  public DatastoreQuartzSchedulerSPI(
      final EventManager eventManager,
      final NodeAccess nodeAccess,
      final Provider<JobStore> jobStoreProvider,
      final Provider<BulkReadScheduler> schedulerProvider,
      final LastShutdownTimeService lastShutdownTimeService,
      final DatabaseStatusDelayedExecutor delayedExecutor,
      @Value("${astrum.quartz.recoverInterruptedJobs:true}") final boolean recoverInterruptedJobs)
  {
    super(eventManager, nodeAccess, jobStoreProvider, schedulerProvider, lastShutdownTimeService, delayedExecutor,
        recoverInterruptedJobs);
  }

  @Override
  protected void doStart() throws Exception {
    super.doStart();

    scheduler.getListenerManager().addSchedulerListener(new SchedulerListenerSupport()
    {
      @Override
      public void jobScheduled(final Trigger trigger) {
        if (!EventHelper.isReplicating()) {
          eventManager.post(new TriggerCreatedEvent(trigger.getKey()));
        }
      }

      @Override
      public void jobUnscheduled(final TriggerKey triggerKey) {
        if (!EventHelper.isReplicating()) {
          eventManager.post(new TriggerDeletedEvent(triggerKey));
        }
      }
    });
  }

  @Guarded(by = STARTED)
  @Override
  public boolean cancel(final String id, final boolean mayInterruptIfRunning) {
    boolean locallyCancelled = super.cancel(id, mayInterruptIfRunning);
    if (locallyCancelled) {
      return true;
    }

    if (!EventHelper.isReplicating()) {
      eventManager.post(new CancelJobEvent(id, mayInterruptIfRunning));
    }

    return false;
  }

  Optional<QuartzTaskJobListener> attachJobListener(final JobKey jobKey) {
    try {
      TriggerKey triggerKey = triggerKey(jobKey.getName(), jobKey.getGroup());
      // getTrigger and/or getJobDetail may return null
      Trigger trigger = scheduler.getTrigger(triggerKey);
      JobDetail jobDetail = scheduler.getJobDetail(jobKey);
      if (trigger != null && jobDetail != null) {
        return Optional.of(attachJobListener(jobDetail, trigger));
      }
      else {
        log.debug("Missing trigger or jobDetail for key {}, trigger: {}, jobDetail: {}", jobKey, trigger, jobDetail);
        return Optional.empty();
      }
    }
    catch (SchedulerException e) {
      log.warn("Unable to attach listener for task '{}' cause {}", jobKey, e.getMessage(),
          log.isDebugEnabled() ? e : null);

      return Optional.empty();
    }
  }

  /*
   * Synchronized to prevent attaching a second listener to a locally created task
   */
  @Override
  protected synchronized QuartzTaskJobListener attachJobListener(
      final JobDetail jobDetail,
      final Trigger trigger) throws SchedulerException
  {
    // Check for an existing listener
    JobListener jobListener = scheduler.getListenerManager().getJobListener(listenerName(jobDetail.getKey()));
    if (jobListener != null) {
      updateJobListener(jobDetail);
      return (QuartzTaskJobListener) jobListener;
    }

    //
    return super.attachJobListener(jobDetail, trigger);
  }

  @Override
  protected Map<JobKey, QuartzTaskInfo> allTasks() throws SchedulerException {
    try (TcclBlock tccl = TcclBlock.begin(this)) {
      return scheduler.getJobKeys(jobGroupEquals(GROUP_NAME))
          .stream()
          .map(this::getOrCreateQuartzTaskInfo)
          .filter(Objects::nonNull)
          .collect(Collectors.toMap(QuartzTaskInfo::getJobKey, Function.identity()));
    }
  }

  @Nullable
  private QuartzTaskInfo getOrCreateQuartzTaskInfo(final JobKey jobKey) {
    QuartzTaskJobListener listener = getExistingListener(jobKey);
    if (listener != null) {
      return listener.getTaskInfo();
    }

    return attachJobListener(jobKey)
        .map(QuartzTaskJobListener::getTaskInfo)
        .orElse(null);
  }

  /*
   * Retrieves an existing listener from the Quartz scheduler's ListenerManager
   */
  @Nullable
  private QuartzTaskJobListener getExistingListener(final JobKey jobKey) {
    try {
      return (QuartzTaskJobListener) scheduler.getListenerManager().getJobListener(listenerName(jobKey));
    }
    catch (SchedulerException e) {
      log.debug("An error occurred retrieving the listener for jobKey {}", jobKey, e);
      return null;
    }
  }

  @Guarded(by = STARTED)
  @Override
  protected QuartzTaskInfo createNewJob(
      final TaskConfiguration config,
      final Schedule schedule) throws SchedulerException
  {
    QuartzTaskInfo taskInfo = super.createNewJob(config, schedule);
    if (!EventHelper.isReplicating()) {
      eventManager.post(new JobCreatedEvent(taskInfo.getJobKey()));
    }
    return taskInfo;
  }

  @Guarded(by = STARTED)
  @Override
  public boolean removeTask(final JobKey jobKey) {
    boolean removed = super.removeTask(jobKey);
    if (removed && !EventHelper.isReplicating()) {
      eventManager.post(new JobDeletedEvent(jobKey));
    }
    return removed;
  }

  @Guarded(by = STARTED)
  @Override
  protected QuartzTaskInfo updateJob(
      final QuartzTaskInfo old,
      final TaskConfiguration config,
      final Schedule schedule) throws SchedulerException
  {
    QuartzTaskInfo taskInfo = super.updateJob(old, config, schedule);
    if (!EventHelper.isReplicating()) {
      eventManager.post(new JobUpdatedEvent(taskInfo.getJobKey()));
    }
    return taskInfo;
  }

  @Subscribe
  public void on(final CancelJobEvent event) {
    if (EventHelper.isReplicating()) {
      log.debug("Received remote request to cancel {}", event.getId());
      cancel(event.getId(), event.isMayInterruptIfRunning());
    }
  }

  @Subscribe
  public void on(final JobCreatedEvent event) {
    handle(event, jobDetail -> {
      attachJobListener(jobDetail.getKey());
      // simulate signals Quartz would have sent
      quartzScheduler.getSchedulerSignaler().signalSchedulingChange(0L);
      quartzScheduler.notifySchedulerListenersJobAdded(jobDetail);
    });
  }

  @Subscribe
  public void on(final JobDeletedEvent event) {
    handle(event, jobDetail -> {
      removeJobListener(jobDetail.getKey());
      // simulate signals Quartz would have sent
      quartzScheduler.getSchedulerSignaler().signalSchedulingChange(0L);
      quartzScheduler.notifySchedulerListenersJobDeleted(jobDetail.getKey());
    });
  }

  @Subscribe
  public void on(final JobUpdatedEvent event) {
    handle(event, jobDetail -> {
      updateJobListener(jobDetail);
      // simulate signals Quartz would have sent
      quartzScheduler.getSchedulerSignaler().signalSchedulingChange(0L);
      quartzScheduler.notifySchedulerListenersJobAdded(jobDetail);
    });
  }

  private void handle(final JobEventSupport event, final Consumer<JobDetail> handler) {
    if (EventHelper.isReplicating()) {
      log.debug("Received {} for {} from node {}", event.getClass(), event.getJobKey(), event.getRemoteNodeId());

      try (TcclBlock tccl = TcclBlock.begin(this)) {
        JobDetail jobDetail = scheduler.getJobDetail(event.getJobKey());
        if (jobDetail == null) {
          log.debug("Missing job for {}", event.getJobKey());
          return;
        }
        synchronized (mutex) {
          handler.accept(jobDetail);
        }
      }
      catch (SchedulerException e) {
        log.warn("Error handling {} for {} cause {}", event.getClass(), event.getJobKey(), e.getMessage(),
            log.isDebugEnabled() ? e : null);
      }
    }
  }

  @Subscribe
  public void on(final TriggerCreatedEvent event) {
    handle(event, trigger -> {
      if (!isRunNow(trigger)) {

        attachJobListener(jobStoreProvider.get().retrieveJob(trigger.getJobKey()), trigger);

        // simulate signals Quartz would have sent
        quartzScheduler.getSchedulerSignaler().signalSchedulingChange(getNextFireMillis(trigger));
        quartzScheduler.notifySchedulerListenersSchduled(trigger);
      }
      else if (isLimitedToThisNode(trigger)) {
        // special "run-now" task which was created on a different node to where it will run
        // when this happens we ping the scheduler to make sure it runs as soon as possible
        quartzScheduler.getSchedulerSignaler().signalSchedulingChange(0L);
        quartzScheduler.notifySchedulerListenersSchduled(trigger);
      }
    });
  }

  @Subscribe
  public void remoteTriggerUpdated(final TriggerUpdatedEvent event) throws SchedulerException {
    handle(event, trigger -> {
      if (!isRunNow(trigger)) {

        updateJobListener(trigger);

        // simulate signals Quartz would have sent
        quartzScheduler.getSchedulerSignaler().signalSchedulingChange(getNextFireMillis(trigger));
        quartzScheduler.notifySchedulerListenersUnscheduled(trigger.getKey());
        quartzScheduler.notifySchedulerListenersSchduled(trigger);
      }
    });
  }

  @Subscribe
  public void remoteTriggerDeleted(final TriggerDeletedEvent event) throws SchedulerException {
    handle(event, trigger -> {
      if (!isRunNow(trigger)) {

        // simulate signals Quartz would have sent
        quartzScheduler.getSchedulerSignaler().signalSchedulingChange(0L);
        quartzScheduler.notifySchedulerListenersUnscheduled(trigger.getKey());

        removeJobListener(trigger.getJobKey());
      }
    });
  }

  private void handle(final TriggerEventSupport event, final Consumer<Trigger> handler) {
    if (EventHelper.isReplicating()) {
      log.debug("Received {} for {} from node {}", event.getClass(), event.getTriggerKey(), event.getRemoteNodeId());

      try (TcclBlock tccl = TcclBlock.begin(this)) {
        Trigger trigger = scheduler.getTrigger(event.getTriggerKey());
        if (trigger == null) {
          log.debug("Missing trigger {}", event.getTriggerKey());
          return;
        }

        synchronized (mutex) {
          handler.accept(trigger);
        }
      }
      catch (SchedulerException e) {
        log.warn("Error handling {} for {} cause {}", event.getClass(), event.getTriggerKey(), e.getMessage(),
            log.isDebugEnabled() ? e : null);
      }
    }
  }

  @FunctionalInterface
  private interface Consumer<E>
  {
    void accept(E entity) throws SchedulerException;
  }
}
