
package org.globalart.astrum.internal.selector;

import org.globalart.astrum.selector.SelectorConfiguration;

/**
 * {@link SelectorConfiguration} event.
 *
 * @since 3.1
 */
public interface SelectorConfigurationEvent
{
  boolean isLocal();

  SelectorConfiguration getSelectorConfiguration();
}
