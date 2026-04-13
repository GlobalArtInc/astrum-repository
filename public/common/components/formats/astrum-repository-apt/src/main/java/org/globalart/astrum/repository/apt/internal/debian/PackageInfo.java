
package org.globalart.astrum.repository.apt.internal.debian;

/**
 * @since 3.17
 */
public class PackageInfo
{
  private static final String PACKAGE_FIELD = "Package";

  private static final String VERSION_FIELD = "Version";

  private static final String ARCHITECTURE_FIELD = "Architecture";

  private final ControlFile controlFile;

  public PackageInfo(final ControlFile controlFile) {
    this.controlFile = controlFile;
  }

  public ControlFile getControlFile() {
    return controlFile;
  }

  public String getPackageName() {
    return getField(PACKAGE_FIELD);
  }

  public String getVersion() {
    return getField(VERSION_FIELD);
  }

  public String getArchitecture() {
    return getField(ARCHITECTURE_FIELD);
  }

  private String getField(final String fieldName) {
    return controlFile.getField(fieldName).map(f -> f.value).get();
  }
}
