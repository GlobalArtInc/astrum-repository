
package org.globalart.astrum.internal.atlas.customizers;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.property.PropertiesFile;
import org.globalart.astrum.supportzip.FileContentSourceSupport;
import org.globalart.astrum.supportzip.SanitizedXmlSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import org.apache.commons.io.IOUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.Arrays.stream;
import static org.globalart.astrum.common.jdbc.JdbcUrlRedactor.redactPassword;
import static org.globalart.astrum.supportzip.PasswordSanitizing.REPLACEMENT;
import static org.globalart.astrum.supportzip.PasswordSanitizing.SENSITIVE_FIELD_NAMES;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.HIGH;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.CONFIG;
import org.springframework.stereotype.Component;

/**
 * Adds installation directory configuration files to support bundle.
 *
 * @since 3.0
 */
@Component
public class InstallConfigurationCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String INSTALL_ETC = "install/etc";

  private static final String WORK_ETC = "work/etc";

  private final ApplicationDirectories applicationDirectories;

  private final String ASTRUM_PROPERTIES = "astrum.properties";

  @Autowired
  public InstallConfigurationCustomizer(final ApplicationDirectories applicationDirectories) {
    this.applicationDirectories = checkNotNull(applicationDirectories);
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    File installDir = applicationDirectories.getInstallDirectory();
    if (installDir != null) {
      File etcDir = new File(installDir, "etc");
      includeFileIfExists(supportBundle, new File(etcDir, "astrum-default.properties"), INSTALL_ETC, HIGH);
      includeFileIfExists(supportBundle, new File(etcDir, ASTRUM_PROPERTIES), INSTALL_ETC, HIGH);
      includeAllFilesInDirIfExists(supportBundle, new File(etcDir, "fabric"), INSTALL_ETC, HIGH);
      includeAllFilesInDirIfExists(supportBundle, new File(etcDir, "jetty"), INSTALL_ETC, HIGH);
      includeAllFilesInDirIfExists(supportBundle, new File(etcDir, "karaf"), INSTALL_ETC, HIGH);
      includeAllFilesInDirIfExists(supportBundle, new File(etcDir, "logback"), INSTALL_ETC, HIGH);
    }

    File workDir = applicationDirectories.getWorkDirectory();
    if (workDir != null) {
      File etcDir = new File(workDir, "etc");
      includeFileIfExists(supportBundle, new File(etcDir, ASTRUM_PROPERTIES), WORK_ETC, HIGH);
      includeAllFilesInDirIfExists(supportBundle, new File(etcDir, "fabric"), WORK_ETC, HIGH);
      includeAllFilesInDirIfExists(supportBundle, new File(etcDir, "logback"), WORK_ETC, HIGH);
    }
  }

  private void includeFileIfExists(
      final SupportBundle supportBundle,
      final File file,
      final String prefixDir,
      final Priority priority)
  {
    if (file != null && file.isFile()) {
      log.debug("Including file: {}", file);
      String fileName = file.getName();
      String filePath = String.join("/", prefixDir, fileName);
      try {
        if (fileName.equals("jetty-https.xml")) {
          supportBundle.add(new SanitizedJettyFileSource(CONFIG, filePath, file, priority));
        }
        else if (fileName.equals("jetty-https-fips.xml")) {
          supportBundle.add(new SanitizedJettyFileSource(CONFIG, filePath, file, priority));
        }
        else if (fileName.endsWith("store.properties")) {
          supportBundle.add(new SanitizedDataStoreFileSource(CONFIG, filePath, file, priority));
        }
        else if (fileName.equals(ASTRUM_PROPERTIES)) {
          supportBundle.add(new SanitizedAstrumFileSource(CONFIG, filePath, file, priority));
        }
        else {
          supportBundle.add(new FileContentSourceSupport(CONFIG, filePath, file, priority));
        }
      }
      catch (IOException e) {
        log.warn("Failed to sanitize {}", file, e);
      }
    }
    else {
      log.warn("Skipping: {}", file);
    }
  }

  private void includeAllFilesInDirIfExists(
      final SupportBundle supportBundle,
      final File directory,
      final String prefixDir,
      final Priority priority)
  {
    if (directory != null && directory.isDirectory()) {
      log.debug("Including dir: {}", directory);
      stream(directory.listFiles()).forEach(
          file -> includeFileIfExists(supportBundle, file, String.join("/", prefixDir, directory.getName()), priority));
    }
    else {
      log.warn("Skipping: {}", directory);
    }
  }

  /**
   * Ad-hoc subclass that encapsulates the XSLT transformation of a Jetty configuration file, removing passwords.
   */
  protected static class SanitizedJettyFileSource
      extends SanitizedXmlSourceSupport
  {
    /**
     * Constructor.
     */
    public SanitizedJettyFileSource(
        final Type type,
        final String path,
        final File file,
        final Priority priority) throws IOException
    {
      super(type, path, file, priority,
          IOUtils.toString(checkNotNull(SanitizedJettyFileSource.class.getResourceAsStream("jetty-stylesheet.xml")),
              UTF_8));
    }
  }

  /**
   * Removes JDBC credentials from *-store.properties, if present.
   */
  protected static class SanitizedDataStoreFileSource
      extends FileContentSourceSupport
  {
    public SanitizedDataStoreFileSource(final Type type, final String path, final File file, final Priority priority) {
      super(CONFIG, path, file, priority);
    }

    @Override
    public InputStream getContent() throws Exception {
      PropertiesFile dataStoreConfiguration = new PropertiesFile(file);
      dataStoreConfiguration.load();
      dataStoreConfiguration.forEach((k, v) -> {
        if (SENSITIVE_FIELD_NAMES.contains(k)) {
          dataStoreConfiguration.replace(k, REPLACEMENT);
        }
        else if ("jdbcUrl".equals(k)) {
          dataStoreConfiguration.put(k, redactPassword((String) v));
        }
      });
      ByteArrayOutputStream outputStream = new ByteArrayOutputStream();
      dataStoreConfiguration.store(outputStream, null);
      return new ByteArrayInputStream(outputStream.toByteArray());
    }
  }

  protected static class SanitizedAstrumFileSource
      extends FileContentSourceSupport
  {
    public SanitizedAstrumFileSource(final Type type, final String path, final File file, final Priority priority) {
      super(CONFIG, path, file, priority);
    }

    @Override
    public InputStream getContent() throws Exception {
      PropertiesFile dataStoreConfiguration = new PropertiesFile(file);
      dataStoreConfiguration.load();
      dataStoreConfiguration.forEach((k, v) -> {
        if (SENSITIVE_FIELD_NAMES.contains(k)) {
          dataStoreConfiguration.replace(k, REPLACEMENT);
        }
        else if ("astrum.datastore.astrum.jdbcUrl".equals(k)) {
          dataStoreConfiguration.put(k, redactPassword((String) v));
        }
      });
      ByteArrayOutputStream outputStream = new ByteArrayOutputStream();
      dataStoreConfiguration.store(outputStream, null);
      return new ByteArrayInputStream(outputStream.toByteArray());
    }
  }
}
