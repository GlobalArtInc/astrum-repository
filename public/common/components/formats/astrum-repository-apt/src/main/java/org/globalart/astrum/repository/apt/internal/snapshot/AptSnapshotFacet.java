
package org.globalart.astrum.repository.apt.internal.snapshot;

import java.io.IOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.view.Content;

/**
 * @since 3.17
 */
@Facet.Exposed
public interface AptSnapshotFacet
    extends Facet
{
  boolean isSnapshotableFile(final String path);

  void createSnapshot(final String id, final SnapshotComponentSelector spec) throws IOException;

  @Nullable
  Content getSnapshotFile(final String id, final String path) throws IOException;

  void deleteSnapshot(final String id) throws IOException;
}
