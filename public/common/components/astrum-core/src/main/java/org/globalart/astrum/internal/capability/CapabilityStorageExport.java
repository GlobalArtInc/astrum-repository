
package org.globalart.astrum.internal.capability;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.internal.capability.storage.CapabilityStorage;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItem;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link CapabilityStorageItem} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("capabilityStorageExport")
public class CapabilityStorageExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final CapabilityStorage capabilityStorage;

  @Autowired
  public CapabilityStorageExport(final CapabilityStorage capabilityStorage) {
    this.capabilityStorage = capabilityStorage;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export CapabilityStorage data to {}", file);
    List<CapabilityStorageItem> capabilities = new ArrayList<>(capabilityStorage.getAll().values());
    exportToJson(capabilities, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring CapabilityStorage data from {}", file);
    importFromJson(file, CapabilityStorageItemData.class).forEach(capabilityStorage::add);
  }
}
