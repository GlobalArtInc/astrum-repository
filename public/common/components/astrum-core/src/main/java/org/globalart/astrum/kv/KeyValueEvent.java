
package org.globalart.astrum.kv;

import org.globalart.astrum.common.event.EventWithSource;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An event fired when a value is set via {@link GlobalKeyValueStore}
 */
public class KeyValueEvent
    extends EventWithSource
{
  private String key;

  private Object value;

  KeyValueEvent() {
    // deserialization
  }

  public KeyValueEvent(final String key, final Object value) {
    this.key = checkNotNull(key);
    this.value = value;
  }

  /**
   * @return the key that was set
   */
  public String getKey() {
    return key;
  }

  /**
   * Note: complex objects will not be preserved for nodes where the event did not originate, for other nodes
   * in the cluster the value will be a Map
   *
   * @return the value, for complex objects in HA
   */
  public Object getValue() {
    return value;
  }

  public void setKey(final String key) {
    this.key = key;
  }

  public void setValue(final Object value) {
    this.value = value;
  }
}
