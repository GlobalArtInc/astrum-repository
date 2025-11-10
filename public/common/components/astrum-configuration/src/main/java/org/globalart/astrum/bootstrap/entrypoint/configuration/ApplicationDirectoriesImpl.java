
package org.globalart.astrum.bootstrap.entrypoint.configuration;

import java.io.File;

import com.google.common.base.Throwables;
import org.springframework.beans.factory.annotation.Autowired;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Default {@link ApplicationDirectories} implementation.
 *
 * @since 2.8
 */
@Component
public class ApplicationDirectoriesImpl
    implements ApplicationDirectories
{
  private static final Logger LOG = LoggerFactory.getLogger(ApplicationDirectoriesImpl.class);

  private final File installDir;

  private final File configDir;

  private final File workDir;

  private final File tempDir;

  private final DirectoryHelper directoryHelper;

  @Autowired
  public ApplicationDirectoriesImpl(
      @Value("${karaf.base}") final String installDir,
      @Value("${karaf.data}") final String workDir,
      final DirectoryHelper directoryHelper)
  {
    // assign this first so it is available for mkdir calls
    this.directoryHelper = directoryHelper;

    this.installDir = resolve(new File(installDir), false);
    LOG.debug("Install dir: {}", this.installDir);

    this.configDir = resolve(new File(installDir, "etc"), false);
    LOG.debug("Config dir: {}", this.configDir);

    this.workDir = resolve(new File(workDir), true);
    LOG.debug("Work dir: {}", this.workDir);

    // Resolve the tmp dir from system properties.
    String tmplocation = System.getProperty("java.io.tmpdir", "tmp");
    this.tempDir = resolve(new File(tmplocation), true);
    LOG.debug("Temp dir: {}", this.tempDir);
  }

  @Override
  public File getInstallDirectory() {
    return installDir;
  }

  @Override
  public File getConfigDirectory(final String subsystem) {
    checkNotNull(subsystem);
    return new File(configDir, subsystem);
  }

  @Override
  public File getTemporaryDirectory() {
    return tempDir;
  }

  @Override
  public File getWorkDirectory() {
    return workDir;
  }

  @Override
  public File getWorkDirectory(final String path, final boolean create) {
    checkNotNull(path);
    File dir = new File(path);
    if (!dir.isAbsolute()) {
      dir = new File(getWorkDirectory(), path);
    }
    return resolve(dir, create);
  }

  @Override
  public File getWorkDirectory(final String path) {
    return getWorkDirectory(path, true);
  }

  private void mkdir(final File dir) {
    if (dir.isDirectory()) {
      // skip already exists
      return;
    }

    try {
      directoryHelper.mkdir(dir.toPath());
      LOG.debug("Created directory: {}", dir);
    }
    catch (Exception e) {
      LOG.error("Failed to create directory: {}", dir);
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }
  }

  private File resolve(File dir, final boolean create) {
    checkNotNull(dir);

    LOG.trace("Resolving directory: {}; create: {}", dir, create);
    try {
      dir = dir.getCanonicalFile();
    }
    catch (Exception e) {
      LOG.error("Failed to canonicalize directory: {}", dir);
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }

    if (create) {
      mkdir(dir);
    }

    return dir;
  }
}
