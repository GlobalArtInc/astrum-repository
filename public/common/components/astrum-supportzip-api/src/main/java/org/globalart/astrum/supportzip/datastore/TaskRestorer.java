
package org.globalart.astrum.supportzip.datastore;

import java.io.File;
import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.supportzip.ImportTaskData;

import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.supportzip.datastore.RestoreHelper.FILE_SUFFIX;

/**
 * Restore Task's related data from JSON file(s).
 *
 * @since 3.30
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class TaskRestorer
    extends StateGuardLifecycleSupport
{
  private final RestoreHelper restoreHelper;

  private final Map<String, ImportTaskData> importTaskByName;

  @Autowired
  public TaskRestorer(
      final RestoreHelper restoreHelper,
      final List<ImportTaskData> importTaskByName)
  {
    this.restoreHelper = checkNotNull(restoreHelper);
    this.importTaskByName = QualifierUtil.buildQualifierBeanMap(checkNotNull(importTaskByName));
  }

  @Override
  protected void doStart() throws Exception {
    maybeRestore();
  }

  private void maybeRestore() throws IOException {
    Path dbDir = restoreHelper.getDbPath();
    for (Entry<String, ImportTaskData> exporterEntry : importTaskByName.entrySet()) {
      String fileName = exporterEntry.getKey() + FILE_SUFFIX;
      File file = dbDir.resolve(fileName).toFile();
      ImportTaskData importTask = exporterEntry.getValue();
      if (file.exists()) {
        importTask.restore(file);
        FileUtils.deleteQuietly(file);
      }
      else {
        log.debug("Can't find {} file to restore data", file);
      }
    }
  }
}
