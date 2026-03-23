
package org.globalart.astrum.repository.maven;

/**
 * Layout policy.
 *
 * @since 3.0
 */
public enum LayoutPolicy
{
  /**
   * Only allow repository paths that are Maven 2 standard layout compliant.
   */
  STRICT,

  /**
   * Allow any repository paths.
   */
  PERMISSIVE
}
