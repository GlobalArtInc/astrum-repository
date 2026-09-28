
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Export/Import {@link KeyStoreData} data to/from JSON file for support zip.
 */
@Component
@Qualifier("keyStoreDataExport")
public class KeyStoreDataExport
    extends JsonExporter
    implements ExportSecurityData, ImportData
{
  private final PersistentKeyStoreStorageManager keyStoreStorageManager;

  @Autowired
  public KeyStoreDataExport(final PersistentKeyStoreStorageManager keyStoreStorageManager) {
    this.keyStoreStorageManager = checkNotNull(keyStoreStorageManager);
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export KeyStoreData to {}", file);
    List<KeyStoreData> keyStoreDataList = keyStoreStorageManager.browse();
    exportToJson(keyStoreDataList, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring KeyStoreData from {}", file);
    List<KeyStoreData> keyStoreDataList = importFromJson(file, KeyStoreData.class);
    for (KeyStoreData keyStoreData : keyStoreDataList) {
      keyStoreStorageManager.save(keyStoreData);
    }
  }
}
