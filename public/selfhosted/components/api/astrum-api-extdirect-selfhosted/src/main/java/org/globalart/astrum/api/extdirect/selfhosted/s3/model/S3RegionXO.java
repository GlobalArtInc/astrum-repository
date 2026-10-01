
package org.globalart.astrum.api.extdirect.selfhosted.s3.model;

/**
 * S3 Region exchange object.
 *
 * @since 3.12
 */
public class S3RegionXO
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

  public S3RegionXO withOrder(final int order) {
    this.order = order;
    return this;
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public S3RegionXO withId(final String id) {
    this.id = id;
    return this;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public S3RegionXO withName(final String name) {
    this.name = name;
    return this;
  }
}
