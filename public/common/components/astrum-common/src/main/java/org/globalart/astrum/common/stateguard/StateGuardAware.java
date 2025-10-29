
package org.globalart.astrum.common.stateguard;

import javax.annotation.Nonnull;

/**
 * Exposes {@link StateGuard}.
 *
 * @since 3.0
 */
public interface StateGuardAware
{
  @Nonnull
  StateGuard getStateGuard();
}
