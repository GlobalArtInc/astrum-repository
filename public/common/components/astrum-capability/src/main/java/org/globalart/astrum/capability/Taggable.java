
package org.globalart.astrum.capability;

import java.util.Set;

/**
 * Implemented by {@link Capability} or {@link CapabilityDescriptor} that supports tagging.
 *
 * @since 2.7
 */
public interface Taggable
{

  /**
   * Returns the tags that this capability is tagged with (could be null).
   */
  Set<Tag> getTags();

}
