
package org.globalart.astrum.internal.email;

import java.io.File;
import java.io.IOException;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link EmailConfiguration} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("emailConfigurationExport")
public class EmailConfigurationExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final EmailConfigurationStore store;

  @Autowired
  public EmailConfigurationExport(final EmailConfigurationStore store) {
    this.store = store;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export EmailConfiguration data to {}", file);
    EmailConfiguration configuration = store.load();
    exportObjectToJson(configuration, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring EmailConfiguration data from {}", file);
    Optional<EmailConfigurationData> configuration = importObjectFromJson(file, EmailConfigurationData.class);
    configuration.ifPresent(store::save);
  }
}
