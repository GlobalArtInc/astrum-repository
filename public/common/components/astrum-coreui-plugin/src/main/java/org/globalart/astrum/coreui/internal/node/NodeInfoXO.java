
package org.globalart.astrum.coreui.internal.node;

/**
 * For transmitting info about nodes in cluster
 */
public class NodeInfoXO
{
  private String name;

  private Boolean local;

  private String displayName;

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public Boolean getLocal() {
    return local;
  }

  public void setLocal(final Boolean local) {
    this.local = local;
  }

  public String getDisplayName() {
    return displayName;
  }

  public void setDisplayName(final String displayName) {
    this.displayName = displayName;
  }

  @Override
  public String toString() {
    return "NodeInfoXO(" +
        "name=" + name +
        ", local=" + local +
        ", displayName=" + displayName +
        ")";
  }
}
