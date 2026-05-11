
package org.globalart.astrum.rapture;

import java.util.Map;

import javax.annotation.Nullable;

/**
 * Implemented by components that provide state values/commands.
 *
 * @since 3.0
 */
public interface StateContributor
{
  /**
   * Returns mapping of state-id to state-value (state-value can be null).
   */
  @Nullable
  Map<String, Object> getState();
}
