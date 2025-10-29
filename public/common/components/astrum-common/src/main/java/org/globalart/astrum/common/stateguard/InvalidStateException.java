
package org.globalart.astrum.common.stateguard;

import java.util.Arrays;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Runtime exception thrown when {@link StateGuard} check fails.
 *
 * @since 3.2.1
 */
public class InvalidStateException
    extends IllegalStateException
{
  private final String invalidState;

  public InvalidStateException(final String invalidState, final String[] allowedStates) {
    super("Invalid state: " + invalidState + "; allowed: " + Arrays.toString(allowedStates));
    this.invalidState = checkNotNull(invalidState);
  }

  public String getInvalidState() {
    return invalidState;
  }
}
