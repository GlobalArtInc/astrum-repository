
package org.globalart.astrum.coreui.search;

/**
 * BrowseableFormat exchange object.
 */
public class BrowseableFormatXO
{
  private String id;

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  @Override
  public String toString() {
    return "BrowseableFormatXO(" +
        "id:" + id +
        ")";
  }
}
