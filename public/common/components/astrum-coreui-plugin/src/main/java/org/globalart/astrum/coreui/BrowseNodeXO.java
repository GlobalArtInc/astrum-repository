
package org.globalart.astrum.coreui;

import java.util.Objects;
import jakarta.validation.constraints.NotBlank;

/**
 * Component exchange object.
 *
 * @since 3.6
 */
public class BrowseNodeXO
{
  @NotBlank
  private String id;

  @NotBlank
  private String text;

  @NotBlank
  private String type;

  private boolean leaf;

  private String componentId;

  private String assetId;

  private String packageUrl;

  public String getId() {
    return id;
  }

  public BrowseNodeXO withId(final String id) {
    this.id = id;
    return this;
  }

  public String getText() {
    return text;
  }

  public BrowseNodeXO withText(final String text) {
    this.text = text;
    return this;
  }

  public String getType() {
    return type;
  }

  public BrowseNodeXO withType(final String type) {
    this.type = type;
    return this;
  }

  public boolean isLeaf() {
    return leaf;
  }

  public BrowseNodeXO withLeaf(final boolean leaf) {
    this.leaf = leaf;
    return this;
  }

  public String getComponentId() {
    return componentId;
  }

  public BrowseNodeXO withComponentId(final String componentId) {
    this.componentId = componentId;
    return this;
  }

  public String getAssetId() {
    return assetId;
  }

  public BrowseNodeXO withAssetId(final String assetId) {
    this.assetId = assetId;
    return this;
  }

  public String getPackageUrl() {
    return packageUrl;
  }

  public BrowseNodeXO withPackageUrl(final String packageUrl) {
    this.packageUrl = packageUrl;
    return this;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BrowseNodeXO other = (BrowseNodeXO) o;
    return Objects.equals(id, other.id);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id);
  }

  @Override
  public String toString() {
    return "BrowseNodeXO{" +
        "id='" + id + '\'' +
        ", text='" + text + '\'' +
        ", type='" + type + '\'' +
        ", leaf=" + leaf +
        ", componentId='" + componentId + '\'' +
        ", assetId='" + assetId + '\'' +
        ", packageUrl='" + packageUrl + '\'' +
        '}';
  }
}
