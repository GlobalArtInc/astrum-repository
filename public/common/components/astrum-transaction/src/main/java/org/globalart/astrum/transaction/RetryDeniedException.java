
package org.globalart.astrum.transaction;

/**
 * {@link Transaction}s can throw this from {@link Transaction#allowRetry} to implicitly deny further retries.
 *
 * @since 3.22
 */
public class RetryDeniedException
    extends RuntimeException
{
  public RetryDeniedException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
