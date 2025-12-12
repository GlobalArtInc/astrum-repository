
package org.globalart.astrum.internal.support;

/**
 * Support Zip exchange object.
 *
 * @since 3.0
 */
public class SupportZipXO
{
  private String file;

  private String name;

  private long size;

  private Boolean truncated;

  public SupportZipXO() {
  }

  public SupportZipXO(String file, String name, long size, Boolean truncated) {
    this.file = file;
    this.name = name;
    this.size = size;
    this.truncated = truncated;
  }

  public String getFile() {
    return file;
  }

  public void setFile(String file) {
    this.file = file;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public long getSize() {
    return size;
  }

  public void setSize(long size) {
    this.size = size;
  }

  public Boolean getTruncated() {
    return truncated;
  }

  public void setTruncated(Boolean truncated) {
    this.truncated = truncated;
  }

  @Override
  public String toString() {
    return "SupportZipXO{" +
        "file='" + file + '\'' +
        ", name='" + name + '\'' +
        ", size='" + size + '\'' +
        ", truncated=" + truncated +
        '}';
  }
}
