
package org.globalart.astrum.common.stateguard;

import javax.annotation.Nullable;

/**
 * Runnable action.
 *
 * @since 3.0
 */
public interface Action<V>
{
  @Nullable
  V run() throws Exception;
}
