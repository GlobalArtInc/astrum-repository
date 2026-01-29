
package org.globalart.astrum.repository.internal.blobstore;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link BlobStoreConfiguration} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("blobStoreConfigurationExport")
public class BlobStoreConfigurationExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final BlobStoreConfigurationStore configurationStore;

  @Autowired
  public BlobStoreConfigurationExport(final BlobStoreConfigurationStore configurationStore) {
    this.configurationStore = configurationStore;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export BlobStoreConfiguration data to {}", file);
    List<BlobStoreConfiguration> configurations = configurationStore.list();
    exportToJson(configurations, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring BlobStoreConfiguration data from {}", file);
    importFromJson(file, BlobStoreConfigurationData.class).forEach(configurationStore::create);
  }
}
