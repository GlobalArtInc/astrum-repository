
package org.globalart.astrum.api.rest.common.blobstore.model;

import java.util.Map;

public class BlobStoreConnectionXO
{
  private String name;

  private String type;

  private Map<String, Map<String, Object>> attributes;

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public Map<String, Map<String, Object>> getAttributes() {
    return attributes;
  }

  public void setAttributes(final Map<String, Map<String, Object>> attributes) {
    this.attributes = attributes;
  }
}
