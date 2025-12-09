
package org.globalart.astrum.internal.httpclient;

import java.io.File;
import java.io.IOException;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link HttpClientConfiguration} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("httpClientConfigurationExport")
public class HttpClientConfigurationExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final HttpClientConfigurationStore store;

  @Autowired
  public HttpClientConfigurationExport(final HttpClientConfigurationStore store) {
    this.store = store;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export HttpClientConfiguration data to {}", file);
    HttpClientConfiguration configuration = store.load();
    exportObjectToJson(configuration, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring HttpClientConfiguration data from {}", file);
    Optional<HttpClientConfigurationData> configuration = importObjectFromJson(file, HttpClientConfigurationData.class);
    configuration.ifPresent(store::save);
  }
}
