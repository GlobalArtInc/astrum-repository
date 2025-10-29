
package org.globalart.astrum.common.stateguard;

import javax.annotation.Nullable;

/**
 * State transition.
 *
 * @since 3.0
 */
public interface Transition
{
  Transition from(String... from);

  @Nullable
  <V> V run(Action<V> action) throws Exception;
}
