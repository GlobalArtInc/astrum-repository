
package org.globalart.astrum.script.plugin.internal;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.script.Script;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link Script} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("scriptExport")
public class ScriptExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final ScriptStore store;

  @Autowired
  public ScriptExport(final ScriptStore store) {
    this.store = store;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export Script data to {}", file);
    List<Script> scripts = store.list();
    exportToJson(scripts, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring Script data from {}", file);
    importFromJson(file, ScriptData.class).forEach(store::create);
  }
}
