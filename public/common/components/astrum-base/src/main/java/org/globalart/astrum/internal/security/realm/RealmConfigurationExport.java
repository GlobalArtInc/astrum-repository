
package org.globalart.astrum.internal.security.realm;

import java.io.File;
import java.io.IOException;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.realm.RealmConfiguration;
import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link RealmConfiguration} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("realmConfigurationExport")
public class RealmConfigurationExport
    extends JsonExporter
    implements ExportSecurityData, ImportData
{
  private final RealmConfigurationStoreImpl configuration;

  @Autowired
  public RealmConfigurationExport(final RealmConfigurationStoreImpl configuration) {
    this.configuration = configuration;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export RealmConfiguration data to {}", file);
    exportObjectToJson(configuration.load(), file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring RealmConfiguration data from {}", file);
    Optional<RealmConfigurationData> realmConfiguration = importObjectFromJson(file, RealmConfigurationData.class);
    realmConfiguration.ifPresent(configuration::save);
  }
}
