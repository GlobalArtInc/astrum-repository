
package org.globalart.astrum.coreui;

/**
 * Blob store group fill policy exchange object.
 *
 * @since 3.14
 */
public class FillPolicyXO
{
  private String id;

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
    return "FillPolicyXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        '}';
  }
}
