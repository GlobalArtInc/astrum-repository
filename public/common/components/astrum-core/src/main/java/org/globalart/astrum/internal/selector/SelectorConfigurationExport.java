
package org.globalart.astrum.internal.selector;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorConfigurationStore;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link SelectorConfiguration} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("selectorConfigurationExport")
public class SelectorConfigurationExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final SelectorConfigurationStore store;

  @Autowired
  public SelectorConfigurationExport(final SelectorConfigurationStore store) {
    this.store = store;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export SelectorConfiguration data to {}", file);
    List<SelectorConfiguration> configurations = store.browse();
    exportToJson(configurations, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring SelectorConfiguration data from {}", file);
    importFromJson(file, SelectorConfigurationData.class).forEach(store::create);
  }
}
