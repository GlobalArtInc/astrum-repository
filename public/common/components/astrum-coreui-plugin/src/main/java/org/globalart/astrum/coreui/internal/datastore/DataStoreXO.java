
package org.globalart.astrum.coreui.internal.datastore;

public class DataStoreXO
{
  private String name;

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  @Override
  public String toString() {
    return "DataStoreXO(" +
        "name:" + name +
        ")";
  }
}
