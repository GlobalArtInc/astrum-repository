
package org.globalart.astrum.repository.r.internal.util;

import java.util.Map;

import org.globalart.astrum.repository.InvalidContentException;

import static org.globalart.astrum.repository.r.internal.RAttributes.P_PACKAGE;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_VERSION;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.archiveExtension;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.archiveFileName;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.fileName;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.isArchive;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.isValidRepoPath;

public final class PackageValidator
{
  public static final String NOT_VALID_PATH_ERROR_MESSAGE =
      "Provided path is not valid and is expecting src/contrib or bin/<os>/contrib/<R_version>";

  public static final String NOT_VALID_EXTENSION_ERROR_MESSAGE = "Provided extension is not .zip, .tar.gz or .tgz";

  private PackageValidator() {
  }

  public static void validateArchiveUploadPath(final String fullPath) {
    if (!isValidRepoPath(fullPath)) {
      throw new InvalidContentException(NOT_VALID_PATH_ERROR_MESSAGE);
    }
    if (!isArchive(fullPath)) {
      throw new InvalidContentException(NOT_VALID_EXTENSION_ERROR_MESSAGE);
    }
  }

  public static void validateArchiveFileName(final String fullPath, final Map<String, String> description) {
    String name = description.get(P_PACKAGE);
    String version = description.get(P_VERSION);
    if (name == null || name.isBlank() || version == null || version.isBlank()) {
      throw new InvalidContentException("DESCRIPTION of " + fileName(fullPath) + " does not declare Package and Version");
    }
    String expected = archiveFileName(name, version, archiveExtension(fullPath));
    if (!expected.equals(fileName(fullPath))) {
      throw new InvalidContentException("File name " + fileName(fullPath) + " does not match package " + name
          + " version " + version + ", expected " + expected);
    }
  }
}
