
package org.globalart.astrum.repository.maven;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Facet;

/**
 * Maven facet, present on all Maven repositories.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface MavenFacet
    extends Facet
{
  /**
   * Returns the format specific {@link MavenPathParser}.
   */
  @Nonnull
  MavenPathParser getMavenPathParser();

  /**
   * Returns the version policy in effect for this repository.
   */
  @Nonnull
  VersionPolicy getVersionPolicy();

  /**
   * Returns the layout policy in effect for this repository.
   */
  LayoutPolicy layoutPolicy();

  /**
   * @since 3.14
   *
   * @param path of the asset to check
   * @return true if it exists
   */
  boolean exists(final MavenPath path);
}
