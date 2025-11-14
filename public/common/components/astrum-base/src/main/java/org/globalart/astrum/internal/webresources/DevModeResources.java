
package org.globalart.astrum.internal.webresources;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

import javax.annotation.Nullable;

import com.google.common.base.Throwables;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Utility related to finding resources when {@code ASTRUM_RESOURCE_DIRS} environment-variable
 * or {@code astrum.resource.dirs} system-property is set.
 *
 * @since 2.7
 */
@Component
public class DevModeResources
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String ENV_VAR = "ASTRUM_RESOURCE_DIRS";

  private static final String SYS_PROP = "astrum.resource.dirs";

  /**
   * List of directories configured as dev-mode resource locations, or null if not configured.
   */
  @Nullable
  private final List<File> resourceLocations;

  public DevModeResources() {
    this.resourceLocations = initializeResourceLocations();
  }

  /**
   * Detect the dev-mode resources search path from environment-variable or system-property, or null if not configured.
   */
  @Nullable
  private String detectSearchPath() {
    String envVar = System.getenv(ENV_VAR);
    String sysProp = System.getProperty(SYS_PROP);

    // complain if both mechanisms to configure are present, this can/will cause confusion
    if (envVar != null && sysProp != null) {
      log.warn("Both environment-variable: {} and system-property: {} are set; environment-variable takes precedence",
          ENV_VAR, SYS_PROP);
    }

    if (envVar != null) {
      log.info("Search-path configured from environment-variable: {}", ENV_VAR);
      return envVar;
    }
    if (sysProp != null) {
      log.info("Search-path configured from system-property: {}", SYS_PROP);
      return sysProp;
    }

    // not configured
    return null;
  }

  @Nullable
  private List<File> initializeResourceLocations() {
    String searchPath = detectSearchPath();

    if (searchPath != null) {
      List<File> locations = new ArrayList<>();
      for (String segment : searchPath.split(",")) {
        String trimmedSegment = segment.trim();
        try {
          File dir = new File(trimmedSegment).getCanonicalFile();
          if (dir.exists() && dir.isDirectory()) {
            locations.add(dir);
          }
          else {
            log.warn("Invalid search-path segment: {}; ignoring", trimmedSegment);
          }
        }
        catch (Exception e) {
          Throwables.throwIfUnchecked(e);
          throw new RuntimeException(e);
        }
      }
      if (!locations.isEmpty()) {
        return locations;
      }
    }
    return null;
  }

  /**
   * Returns list of detected dev-mode resource locations or null if not configured or not valid locations detected.
   *
   * @since 3.0
   */
  @Nullable
  public List<File> getResourceLocations() {
    return resourceLocations;
  }

  /**
   * Returns a file reference for given path if dev-mode is configured and a matching file exists.
   */
  @Nullable
  public File getFileIfOnFileSystem(final String path) {
    if (resourceLocations != null) {
      for (File dir : resourceLocations) {
        File file = new File(dir, path);
        if (file.exists()) {
          return file;
        }
      }
    }
    return null;
  }
}
