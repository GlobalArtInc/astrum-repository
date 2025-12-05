
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.time.Instant;
import java.time.ZonedDateTime;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Iterator;
import java.util.List;
import java.util.Optional;
import org.globalart.astrum.logging.task.TaskLogHome;
import org.globalart.astrum.supportzip.FileContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.time.Instant.ofEpochMilli;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.DEFAULT;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.REPLICATIONLOG;
import org.springframework.stereotype.Component;

/**
 * Class to add replication v2 logs to support bundle
 */
@Component
public class ReplicationLogCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public void customize(final SupportBundle supportBundle) {
    Instant cutOff = ZonedDateTime.now().minusHours(24).toInstant();

    getReplicationLogsHome()
        .map(File::new)
        .map(f -> getChildren(f, Collections.singletonList("log"), false))
        .ifPresent(
            iterator -> iterator.forEachRemaining(file -> this.updateSupportBundle(file, supportBundle, cutOff)));
  }

  private Iterator<File> getChildren(final File folder, final List<String> extensions, final boolean recursive) {
    List<File> validFiles = new ArrayList<>();

    if (folder.exists()) {
      File[] children = folder.listFiles();

      for (File child : children) {
        if (child.isDirectory() && recursive) {
          Iterator<File> childrenResults = getChildren(child, extensions, recursive);
          childrenResults.forEachRemaining(validFiles::add);
        }
        else if (isValidFileExtension(child, extensions)) {
          validFiles.add(child);
        }
      }
    }

    return validFiles.iterator();
  }

  private boolean isValidFileExtension(final File child, final List<String> extensions) {
    String fileName = child.getName();
    int idx = fileName.lastIndexOf(".");

    if (idx > 0) {
      String extension = fileName.substring(idx + 1);
      return extensions.contains(extension);
    }

    return false;
  }

  private void updateSupportBundle(final File file, final SupportBundle supportBundle, final Instant cutOff) {
    if (ofEpochMilli(file.lastModified()).isAfter(cutOff)) {
      log.debug("adding replication log file '{}'", file);
      supportBundle.add(
          new FileContentSourceSupport(REPLICATIONLOG, String.format("log/replication/%s", file.getName()), file,
              DEFAULT));
    }
    else {
      log.debug("Skipping replication log file [past 24 hours]: {}", file);
    }
  }

  @VisibleForTesting
  Optional<String> getReplicationLogsHome() {
    return TaskLogHome.getReplicationLogsHome();
  }
}
