
package org.globalart.astrum.repository.apt.internal.snapshot;

import java.util.List;

/**
 * @since 3.17
 */
import org.globalart.astrum.repository.apt.internal.debian.Release;

public interface SnapshotComponentSelector
{
  List<String> getArchitectures(final Release release);

  List<String> getComponents(final Release release);
}
