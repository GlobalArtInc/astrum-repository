
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.time.Instant;
import java.time.ZonedDateTime;
import org.globalart.astrum.logging.task.TaskLogHome;
import org.globalart.astrum.supportzip.FileContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.time.Instant.ofEpochMilli;
import static org.apache.commons.io.FileUtils.iterateFiles;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.DEFAULT;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.TASKLOG;
import org.springframework.stereotype.Component;

/**
 * Adds log files to support bundle.
 *
 * @since 3.5
 */
@Component
public class TaskLogCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String[] extensions = new String[]{"log"};

  @Override
  public void customize(final SupportBundle supportBundle) {
    Instant cutoff = ZonedDateTime.now().minusHours(24).toInstant();
    String taskLogHome = getTaskLogHome();
    if (taskLogHome != null) {
      iterateFiles(new File(taskLogHome), extensions, false).forEachRemaining(file -> {
        if (ofEpochMilli(file.lastModified()).isAfter(cutoff)) {
          supportBundle.add(
              new FileContentSourceSupport(TASKLOG, String.format("log/tasks/%s", file.getName()), file, DEFAULT));
        }
        else {
          log.debug("Skipping file [past 24 hours]: {}", file);
        }
      });
    }
  }

  @VisibleForTesting
  protected String getTaskLogHome() {
    return TaskLogHome.getTaskLogsHome();
  }
}
