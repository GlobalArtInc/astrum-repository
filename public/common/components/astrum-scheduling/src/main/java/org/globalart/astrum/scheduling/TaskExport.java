
package org.globalart.astrum.scheduling;

import java.io.File;
import java.io.IOException;
import java.util.Date;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.Future;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.scheduling.schedule.ScheduleDeserializer;
import org.globalart.astrum.scheduling.schedule.ScheduleSerializer;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportTaskData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.fasterxml.jackson.databind.annotation.JsonSerialize;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link TaskInfo} data to/from a JSON file.
 *
 * @since 3.30
 */
@Component
@Qualifier("taskExport")
public class TaskExport
    extends JsonExporter
    implements ExportConfigData, ImportTaskData
{
  private final TaskScheduler taskScheduler;

  @Autowired
  public TaskExport(final TaskScheduler taskScheduler) {
    this.taskScheduler = taskScheduler;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export Tasks data to {}", file);
    List<TaskInfo> tasks = taskScheduler.listsTasks();
    exportToJson(tasks, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring Tasks data from {}", file);

    // remove all tasks by default
    List<TaskInfo> defaultTasks = taskScheduler.listsTasks();
    defaultTasks.forEach(TaskInfo::remove);

    List<TaskInfoData> tasks = importFromJson(file, TaskInfoData.class);
    tasks.forEach(task -> taskScheduler.scheduleTask(task.getConfiguration(), task.getSchedule()));
  }

  // this class is used only for Serialization/Deserialization
  @JsonIgnoreProperties(ignoreUnknown = true)
  public static class TaskInfoData
      implements TaskInfo
  {
    private TaskConfiguration configuration;

    @JsonSerialize(using = ScheduleSerializer.class)
    @JsonDeserialize(using = ScheduleDeserializer.class)
    private Schedule schedule;

    private Object lastResult;

    private final Map<String, Object> context = new HashMap<>();

    @Override
    public String getId() {
      return configuration.getId();
    }

    @Override
    public String getName() {
      return configuration.getName();
    }

    @Override
    public String getTypeId() {
      return configuration.getTypeId();
    }

    @Override
    public String getMessage() {
      return configuration.getMessage();
    }

    @Override
    public TaskConfiguration getConfiguration() {
      return configuration;
    }

    public void setConfiguration(final TaskConfiguration configuration) {
      this.configuration = configuration;
    }

    @Override
    public Schedule getSchedule() {
      return schedule;
    }

    public void setSchedule(final Schedule schedule) {
      this.schedule = schedule;
    }

    @Override
    public CurrentState getCurrentState() {
      return new CurrentStateData();
    }

    @Override
    public Object getLastResult() {
      return lastResult;
    }

    @Override
    public LastRunState getLastRunState() {
      return configuration.getLastRunState();
    }

    @Override
    public boolean remove() {
      return true;
    }

    @Override
    public TaskInfo runNow(final String triggerSource) {
      return this;
    }

    @Override
    public String getTriggerSource() {
      return null;
    }

    @Override
    public Map<String, Object> getContext() {
      return context;
    }
  }

  // this class is used only for Serialization/Deserialization
  public static class CurrentStateData
      implements CurrentState
  {
    private TaskState state;

    private Date nextRun;

    @Override
    public TaskState getState() {
      return state;
    }

    @Override
    public Date getNextRun() {
      return nextRun;
    }

    @Override
    public Date getRunStarted() {
      return null;
    }

    @Override
    public TaskState getRunState() {
      return null;
    }

    @Override
    public Future<?> getFuture() {
      return null;
    }
  }
}
