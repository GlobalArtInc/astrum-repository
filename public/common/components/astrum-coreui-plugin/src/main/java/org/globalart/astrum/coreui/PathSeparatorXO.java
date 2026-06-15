
package org.globalart.astrum.coreui;

import jakarta.validation.constraints.NotEmpty;

/**
 * @since 3.0
 */
public class PathSeparatorXO
{
  @NotEmpty
  private String path;

  @NotEmpty
  private String fileSeparator;

  public String getPath() {
    return path;
  }

  public void setPath(String path) {
    this.path = path;
  }

  public String getFileSeparator() {
    return fileSeparator;
  }

  public void setFileSeparator(String fileSeparator) {
    this.fileSeparator = fileSeparator;
  }
}
