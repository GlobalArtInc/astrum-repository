
package org.globalart.astrum.tasklog;

import java.io.File;
import java.io.IOException;
import java.time.LocalDate;
import java.time.ZoneId;
import java.util.Date;
import java.util.Iterator;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.logging.task.TaskLogHome;

import com.google.common.annotations.VisibleForTesting;
import org.apache.commons.io.filefilter.AgeFileFilter;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.apache.commons.io.FileUtils.forceDelete;
import static org.apache.commons.io.FileUtils.iterateFiles;
import org.springframework.stereotype.Component;

/**
 * Cleanup the task log files based on how old in days they are
 *
 * @since 3.5
 */
@Component
public class TaskLogCleanup
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Integer numberOfDays;

  private String taskLogHome;

  @Autowired
  public TaskLogCleanup(
      @Value("${astrum.tasks.log.cleanup.numberOfDays:30}") final Integer numberOfDays)
  {
    this.numberOfDays = checkNotNull(numberOfDays);
  }

  void cleanup() {
    String taskLogsHome = getTaskLogHome();

    if (taskLogsHome == null) {
      // we are forgiving if the task logs home is not defined. Just log a message with a call to action.
      log.warn("Unable to cleanup task log files. Please check that the 'tasklogfile' appender exists in logback.xml");
      return;
    }

    File logFilesHome = new File(taskLogsHome);

    log.info("Cleaning up log files in {} older than {} days", logFilesHome.getAbsolutePath(), numberOfDays);

    LocalDate now = LocalDate.now().minusDays(numberOfDays);
    Date thresholdDate = Date.from(now.atStartOfDay(ZoneId.systemDefault()).toInstant());
    AgeFileFilter ageFileFilter = new AgeFileFilter(thresholdDate);
    Iterator<File> filesToDelete = iterateFiles(logFilesHome, ageFileFilter, ageFileFilter);
    filesToDelete.forEachRemaining(f -> {
      try {
        forceDelete(f);
        log.info("Removed task log file {}", f.toString());
      }
      catch (IOException e) { // NOSONAR
        log.error("Unable to delete task file {}. Message was {}.", f.toString(), e.getMessage());
      }
    });
  }

  @VisibleForTesting
  String getTaskLogHome() {
    if (taskLogHome == null) {
      taskLogHome = TaskLogHome.getTaskLogsHome();
    }
    return taskLogHome;
  }
}
