
package org.globalart.astrum.repository.maven;

import java.io.IOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;

import org.joda.time.DateTime;

/**
 * Maven2 specific index facet responsible to generate index (for hosted and group repositories).
 *
 * @since 3.0
 */
@Facet.Exposed
public interface MavenIndexFacet
    extends Facet
{
  /**
   * Returns time when index was last published on this repository, or {@code null} if index is not published for
   * whatever reason.
   */
  @Nullable
  DateTime lastPublished() throws IOException;

  /**
   * Publishes Maven Indexer indexes repository for downstream consumption.
   */
  void publishIndex() throws IOException;

  /**
   * Removes published Maven Indexer indexes from repository (or cache, if proxy).
   */
  void unpublishIndex() throws IOException;
}
