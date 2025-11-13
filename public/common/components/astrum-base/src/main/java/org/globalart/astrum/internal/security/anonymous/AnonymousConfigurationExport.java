
package org.globalart.astrum.internal.security.anonymous;

import java.io.File;
import java.io.IOException;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link AnonymousConfiguration} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("anonymousConfigurationExport")
public class AnonymousConfigurationExport
    extends JsonExporter
    implements ExportSecurityData, ImportData
{
  private final AnonymousConfigurationStore anonymousConfigurationStore;

  @Autowired
  public AnonymousConfigurationExport(final AnonymousConfigurationStore anonymousConfigurationStore) {
    this.anonymousConfigurationStore = anonymousConfigurationStore;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export AnonymousConfiguration data to {}", file);
    AnonymousConfiguration configuration = anonymousConfigurationStore.load();
    exportObjectToJson(configuration, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring AnonymousConfiguration data from {}", file);
    Optional<AnonymousConfigurationData> configuration = importObjectFromJson(file, AnonymousConfigurationData.class);
    configuration.ifPresent(anonymousConfigurationStore::save);
  }
}
