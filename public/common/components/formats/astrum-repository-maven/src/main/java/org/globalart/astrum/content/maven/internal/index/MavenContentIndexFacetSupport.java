
package org.globalart.astrum.content.maven.internal.index;

import java.io.IOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.maven.MavenIndexFacet;
import org.globalart.astrum.repository.maven.internal.MavenIndexPublisher;

import org.joda.time.DateTime;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link MavenIndexFacet} support.
 *
 * @since 3.26
 */
public abstract class MavenContentIndexFacetSupport
    extends FacetSupport
    implements MavenIndexFacet
{

  final MavenIndexPublisher mavenIndexPublisher;

  protected MavenContentIndexFacetSupport(final MavenIndexPublisher mavenIndexPublisher) {
    this.mavenIndexPublisher = checkNotNull(mavenIndexPublisher);
  }

  @Nullable
  @Override
  public DateTime lastPublished() throws IOException {
    return mavenIndexPublisher.lastPublished(getRepository());
  }

  @Override
  public void unpublishIndex() throws IOException {
    mavenIndexPublisher.unpublishIndexFiles(getRepository());
  }
}
