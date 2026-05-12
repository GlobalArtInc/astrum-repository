
package org.globalart.astrum.rapture.internal.state;

/**
 * State value exchange object.
 *
 * @since 3.0
 */
public class StateValueXO
{
  private String hash;

  private Object value;

  public StateValueXO() {
  }

  public StateValueXO(String hash, Object value) {
    this.hash = hash;
    this.value = value;
  }

  public String getHash() {
    return hash;
  }

  public void setHash(String hash) {
    this.hash = hash;
  }

  public Object getValue() {
    return value;
  }

  public void setValue(Object value) {
    this.value = value;
  }

  @Override
  public String toString() {
    return "StateValueXO{" +
        "hash='" + hash + '\'' +
        ", value=" + value +
        '}';
  }
}
