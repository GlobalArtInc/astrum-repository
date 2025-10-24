
package org.globalart.astrum.common.cooperation2;

/**
 * Thrown when the current thread cannot cooperate.
 *
 * This may be due to an interrupt, timeout, or reaching a thread limit.
 *
 * @since 3.41
 */
public class CooperationException
    extends RuntimeException
{
  public CooperationException(final String message) {
    super(message);
  }
}
