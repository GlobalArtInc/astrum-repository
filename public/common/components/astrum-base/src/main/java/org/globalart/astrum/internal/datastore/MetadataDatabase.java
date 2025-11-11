
package org.globalart.astrum.internal.datastore;

import java.io.File;
import java.io.FileOutputStream;
import java.io.OutputStream;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;
import java.util.Properties;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.api.DataStoreManager;
import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Add {@link SupportBundle} to export information about database.
 *
 * @since 3.30
 */
@Component
public class MetadataDatabase
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final Path PATH = Paths.get("work");

  private static final String DB_INFO_FILE_NAME = "db_info.properties";

  private final DataStoreManager dataStoreManager;

  @Autowired
  public MetadataDatabase(final DataStoreManager dataStoreManager) {
    this.dataStoreManager = checkNotNull(dataStoreManager);
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    supportBundle.add(getDbInfo(DB_INFO_FILE_NAME));
  }

  private GeneratedContentSourceSupport getDbInfo(final String fileName) {
    return new GeneratedContentSourceSupport(Type.CONFIG, PATH.resolve(fileName).toString())
    {
      @Override
      protected void generate(final File file) throws Exception {
        Optional<DataStore<?>> dataStore = dataStoreManager.get(DataStoreManager.DEFAULT_DATASTORE_NAME);
        if (dataStore.isPresent()) {
          try (OutputStream output = new FileOutputStream(file)) {
            Properties dsProperties = getDbInfo(dataStore.get());
            dsProperties.store(output, null);
          }
        }
      }
    };
  }

  private Properties getDbInfo(final DataStore<?> dataStore) {
    Properties dsProperties = new Properties();
    try (Connection connection = dataStore.getDataSource().getConnection()) {
      dsProperties.setProperty("DatabaseProductName", connection.getMetaData().getDatabaseProductName());
      dsProperties.setProperty("DatabaseProductVersion", connection.getMetaData().getDatabaseProductVersion());
    }
    catch (SQLException e) {
      log.error("Can't collect datastore information", e);
    }

    return dsProperties;
  }
}
