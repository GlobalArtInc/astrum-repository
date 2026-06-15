
package org.globalart.astrum.coreui.internal.log;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import org.globalart.astrum.common.log.LogManager;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;

/**
 * Log information object
 *
 * @since 3.39
 */
public class LogXO
{
  private final Logger log = LoggerFactory.getLogger(getClass().getName());

  private String fileName = null;

  private long size = -1;

  private long lastModified = -1;

  public LogXO(Path path) {
    checkNotNull(path);
    try {
      if (LogManager.TASKS_PREFIX.startsWith(path.getParent().getFileName().toString())) {
        this.fileName = LogManager.TASKS_PREFIX + path.getFileName().toString();
      }
      else if (LogManager.REPLICATION_PREFIX.startsWith(path.getParent().getFileName().toString())) {
        this.fileName = LogManager.REPLICATION_PREFIX + path.getFileName().toString();
      }
      else {
        this.fileName = path.getFileName().toString();
      }
      this.size = Files.size(path);
      this.lastModified = Files.getLastModifiedTime(path).toMillis();
    }
    catch (IOException e) {
      log.debug(format("Unable to get information about log file at {%s}", path));
    }
  }

  public String getFileName() {
    return fileName;
  }

  public long getSize() {
    return size;
  }

  public long getLastModified() {
    return lastModified;
  }
}
