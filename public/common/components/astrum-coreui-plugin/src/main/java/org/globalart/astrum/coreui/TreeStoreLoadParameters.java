
package org.globalart.astrum.coreui;

/**
 * Ext TreeStore load parameters.
 *
 * @since 3.6
 */
public class TreeStoreLoadParameters
{
  private String node;

  private String repositoryName;

  private String filter;

  public String getNode() {
    return node;
  }

  public void setNode(final String node) {
    this.node = node;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public void setRepositoryName(final String repositoryName) {
    this.repositoryName = repositoryName;
  }

  public String getFilter() {
    return filter;
  }

  public void setFilter(final String filter) {
    this.filter = filter;
  }

  @Override
  public String toString() {
    return "TreeStoreLoadParameters{" +
        "node=" + node +
        "repositoryName=" + repositoryName +
        "filter=" + filter +
        '}';
  }
}
