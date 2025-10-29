
package org.globalart.astrum.common.stateguard;

import javax.annotation.Nullable;

/**
 * Guarded state.
 *
 * @since 3.0
 */
public interface Guard
{
  @Nullable
  <V> V run(Action<V> action) throws Exception;
}
