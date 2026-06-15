
package org.globalart.astrum.coreui;

/**
 * Repository reference exchange object.
 *
 * @since 3.0
 */
public class RepositoryReferenceXO
    extends ReferenceXO
{
  private String type;

  private String format;

  private String versionPolicy;

  private String url;

  private String blobStoreName;

  private RepositoryStatusXO status;

  /**
   * sortOrder will override the typical alphanumeric ordering in the UI, so the higher your sortOrder, the closer to
   * the top you will get
   */
  private int sortOrder = 0;

  public RepositoryReferenceXO(
      final String id,
      final String name,
      final String type,
      final String format,
      final String versionPolicy,
      final String url,
      final String blobStoreName,
      final RepositoryStatusXO status,
      final int sortOrder)
  {
    this(id, name, type, format, versionPolicy, url, blobStoreName, status);
    this.sortOrder = sortOrder;
  }

  public RepositoryReferenceXO(
      final String id,
      final String name,
      final String type,
      final String format,
      final String versionPolicy,
      final String url,
      final String blobStoreName,
      final RepositoryStatusXO status)
  {
    setId(id);
    setName(name);
    this.type = type;
    this.format = format;
    this.versionPolicy = versionPolicy;
    this.url = url;
    this.status = status;
    this.blobStoreName = blobStoreName;
  }

  public String getType() {
    return type;
  }

  public String getFormat() {
    return format;
  }

  public String getVersionPolicy() {
    return versionPolicy;
  }

  public String getUrl() {
    return url;
  }

  public String getBlobStoreName() {
    return blobStoreName;
  }

  public RepositoryStatusXO getStatus() {
    return status;
  }

  public int getSortOrder() {
    return sortOrder;
  }

  @Override
  public String toString() {
    return "RepositoryReferenceXO [id=" + getId() + ", name=" + getName() + ", type=" + type + ", format=" + format +
        ", versionPolicy=" + versionPolicy + ", url=" + url + ", blobStoreName=" + blobStoreName + ", status=" + status
        + ", sortOrder=" + sortOrder + "]";
  }
}
