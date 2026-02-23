
package org.globalart.astrum.repository.search;

/**
 * Browses the distinct versions of a single component, one page at a time.
 */
public interface ComponentVersionSearch
{
  /**
   * @param query the component coordinates, filter, and page bounds
   * @return the requested page and the total distinct version count
   */
  ComponentVersionPage browseVersions(ComponentVersionQuery query);
}
