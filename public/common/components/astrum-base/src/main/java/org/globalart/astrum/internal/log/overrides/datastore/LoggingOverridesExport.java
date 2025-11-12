
package org.globalart.astrum.internal.log.overrides.datastore;

import java.io.File;
import java.io.IOException;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED;

/**
 * Write/Read {@link LoggingOverridesData} to/from a JSON file for support zip export.
 */
@Component
@Qualifier("loggingOverridesExport")
@ConditionalOnProperty(name = DATASTORE_CLUSTERED_ENABLED, havingValue = "true")
public class LoggingOverridesExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final LoggingOverridesStore store;

  @Autowired
  public LoggingOverridesExport(final LoggingOverridesStore store) {
    this.store = store;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export LoggingOverrides data to {}", file);
    List<LoggingOverridesData> records = StreamSupport.stream(store.readRecords().spliterator(), false)
        .collect(Collectors.toList());
    exportToJson(records, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring LoggingOverrides data from {}", file);
    importFromJson(file, LoggingOverridesData.class).forEach(store::create);
  }
}
