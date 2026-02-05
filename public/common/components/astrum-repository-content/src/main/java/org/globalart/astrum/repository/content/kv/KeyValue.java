
package org.globalart.astrum.repository.content.kv;

import java.time.OffsetDateTime;

import org.globalart.astrum.common.entity.ContinuationAware;

public class KeyValue
    implements ContinuationAware
{
  Integer keyValueId;

  private String category;

  private String key;

  private String value;

  private OffsetDateTime created;

  public String getCategory() {
    return category;
  }

  public void setCategory(final String category) {
    this.category = category;
  }

  public String getKey() {
    return key;
  }

  public void setKey(final String key) {
    this.key = key;
  }

  public String getValue() {
    return value;
  }

  public void setValue(final String value) {
    this.value = value;
  }

  @Override
  public String nextContinuationToken() {
    return key;
  }

  @Override
  public String toString() {
    return "KeyValue [keyValueId=" + keyValueId + ", key=" + key + ", value=" + value + "]";
  }

  public OffsetDateTime getCreated() {
    return created;
  }

  public void setCreated(OffsetDateTime created) {
    this.created = created;
  }
}
