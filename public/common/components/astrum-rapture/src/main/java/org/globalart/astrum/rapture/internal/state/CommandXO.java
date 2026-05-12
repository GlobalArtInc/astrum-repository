
package org.globalart.astrum.rapture.internal.state;

/**
 * Command exchange object.
 *
 * @since 3.0
 */
public class CommandXO
{
  private String type;

  private Object data;

  public CommandXO() {
  }

  public CommandXO(String type, Object data) {
    this.type = type;
    this.data = data;
  }

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
  }

  public Object getData() {
    return data;
  }

  public void setData(Object data) {
    this.data = data;
  }

  @Override
  public String toString() {
    return "CommandXO{" +
        "type='" + type + '\'' +
        ", data=" + data +
        '}';
  }
}
