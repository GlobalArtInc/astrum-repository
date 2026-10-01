
package org.globalart.astrum.api.extdirect.selfhosted.s3.model;

/**
 * S3 Encryption exchange object.
 *
 * @since 3.19
 */
public class S3EncryptionTypeXO
{
  private int order;

  private String id;

  private String name;

  public int getOrder() {
    return order;
  }

  public void setOrder(final int order) {
    this.order = order;
  }

  public S3EncryptionTypeXO withOrder(final int order) {
    this.order = order;
    return this;
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public S3EncryptionTypeXO withId(final String id) {
    this.id = id;
    return this;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public S3EncryptionTypeXO withName(final String name) {
    this.name = name;
    return this;
  }
}
