
package org.globalart.astrum.security.internal;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.security.config.AdminPasswordFileManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * @since 3.17
 */
@Component
public class AdminPasswordFileManagerImpl
    implements AdminPasswordFileManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String FILENAME = "admin.password";

  public final ApplicationDirectories applicationDirectories;

  private final File adminPasswordFile;

  @Autowired
  public AdminPasswordFileManagerImpl(final ApplicationDirectories applicationDirectories) {
    this.applicationDirectories = checkNotNull(applicationDirectories);
    adminPasswordFile = new File(applicationDirectories.getWorkDirectory(), FILENAME);
  }

  @Override
  public boolean writeFile(String password) throws IOException {
    File workdir = applicationDirectories.getWorkDirectory();
    if (!workdir.isDirectory() && !workdir.mkdirs()) {
      log.error("Failed to create work directory {}", workdir);
      return false;
    }

    if (adminPasswordFile.createNewFile()) {
      adminPasswordFile.setReadable(true, true);
    }

    try {
      log.info("Writing admin user temporary password to {}", adminPasswordFile.toString());
      Files.write(adminPasswordFile.toPath(), password.getBytes(StandardCharsets.UTF_8));
    }
    catch (Exception e) {
      log.error("Failed to write temporary password to disk", e);
      return false;
    }

    return true;
  }

  @Override
  public boolean exists() {
    return adminPasswordFile.exists();
  }

  @Override
  public String getPath() {
    return adminPasswordFile.getAbsolutePath();
  }

  @Override
  public String readFile() throws IOException {
    if (adminPasswordFile.exists()) {
      return new String(Files.readAllBytes(adminPasswordFile.toPath()), StandardCharsets.UTF_8);
    }

    return null;
  }

  @Override
  public void removeFile() {
    if (adminPasswordFile.exists() && !adminPasswordFile.delete()) {
      log.error("Failed to delete admin.password file {}", adminPasswordFile);
    }
  }
}
