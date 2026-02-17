
package org.globalart.astrum.repository.browse.node;

/**
 * @since 3.6
 */
public class BrowseListItem
{
  private final String resourceUri;

  private final String name;

  private final boolean collection;

  private final String lastModified;

  private final String size;

  private final String description;

  public BrowseListItem(
      final String resourceUri,
      final String name,
      final boolean collection,
      final String lastModified,
      final String size,
      final String description)
  {
    this.resourceUri = resourceUri;
    this.name = name;
    this.collection = collection;
    this.lastModified = lastModified;
    this.size = size;
    this.description = description;
  }

  public String getResourceUri() {
    return resourceUri;
  }

  public String getName() {
    return name;
  }

  public boolean isCollection() {
    return collection;
  }

  public String getLastModified() {
    return lastModified;
  }

  public String getSize() {
    return size;
  }

  public String getDescription() {
    return description;
  }
}
