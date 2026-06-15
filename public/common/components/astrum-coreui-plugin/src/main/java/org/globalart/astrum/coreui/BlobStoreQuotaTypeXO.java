
package org.globalart.astrum.coreui;

import jakarta.validation.constraints.NotBlank;

/**
 * BlobStore Quota Type exchange object.
 *
 * @since 3.14
 */
public class BlobStoreQuotaTypeXO
{
  @NotBlank
  private String id;

  @NotBlank
  private String name;

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  @Override
  public String toString() {
    return "BlobStoreQuotaTypeXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        '}';
  }
}
