
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
import org.globalart.astrum.supportzip.ImportData;

import org.apache.commons.io.FileUtils;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.RESTORE;
import static org.globalart.astrum.supportzip.datastore.RestoreHelper.FILE_SUFFIX;
import org.springframework.stereotype.Component;

/**
 * Restore {@link org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type#CONFIG} and
 * {@link org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type#SECURITY} data from JSON files.
 *
 * @since 3.29
 */
@Component
@Order(Ordered.HIGHEST_PRECEDENCE)
@ManagedLifecycle(phase = RESTORE)
public class SupportRestorer
    extends StateGuardLifecycleSupport
{
  private final RestoreHelper restoreHelper;

  private final Map<String, ImportData> importDataByName;

  @Autowired
  public SupportRestorer(
      final RestoreHelper restoreHelper,
      final List<ImportData> importDataByNameList)
  {
    this.restoreHelper = checkNotNull(restoreHelper);
    this.importDataByName = QualifierUtil.buildQualifierBeanMap(checkNotNull(importDataByNameList));
  }

  @Override
  protected void doStart() throws Exception {
    maybeRestore();
  }

  private void maybeRestore() throws IOException {
    Path dbDir = restoreHelper.getDbPath();
    for (Entry<String, ImportData> importerEntry : importDataByName.entrySet()) {
      String fileName = importerEntry.getKey() + FILE_SUFFIX;
      File file = dbDir.resolve(fileName).toFile();
      ImportData importer = importerEntry.getValue();
      if (file.exists()) {
        importer.restore(file);
        FileUtils.deleteQuietly(file);
      }
      else {
        log.debug("Can't find {} file to restore data", file);
      }
    }
  }
}
