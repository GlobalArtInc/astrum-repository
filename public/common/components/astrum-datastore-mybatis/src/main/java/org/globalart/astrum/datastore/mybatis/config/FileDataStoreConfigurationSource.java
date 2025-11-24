package org.globalart.astrum.datastore.mybatis.config;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(FileDataStoreConfigurationSource.NAME)
@Order(FileDataStoreConfigurationSource.ORDER_FILE)
public class FileDataStoreConfigurationSource
    extends ExternalDataStoreConfigurationSource
{
  static final String NAME = "file";

  static final String FILE_NAME = "-store.properties";

  private final ApplicationDirectories directories;

  @Autowired
  public FileDataStoreConfigurationSource(final ApplicationDirectories directories) {
    this.directories = checkNotNull(directories);
  }

  @Override
  public String getName() {
    return "File";
  }

  @Override
  protected String qualifier() {
    return NAME;
  }

  @Override
  protected String lookup(final String storeName, final String attribute) {
    return read(storeName).getProperty(attribute);
  }

  Path file(final String storeName) {
    return directories.getWorkDirectory("etc/fabric").toPath().resolve(storeName + FILE_NAME);
  }

  private Properties read(final String storeName) {
    Properties properties = new Properties();
    Path path = file(storeName);
    if (Files.isReadable(path)) {
      try (InputStream input = Files.newInputStream(path)) {
        properties.load(input);
      }
      catch (IOException e) {
        log.warn("Could not read data store configuration {}: {}", path, e.getMessage());
      }
    }
    return properties;
  }
}
