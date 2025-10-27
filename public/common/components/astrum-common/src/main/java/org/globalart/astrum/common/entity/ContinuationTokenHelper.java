
package org.globalart.astrum.common.entity;

import javax.annotation.Nullable;

/**
 * Helper to provide an ID from a continuation token
 *
 * @since 3.7
 */
public interface ContinuationTokenHelper
{
  @Nullable
  String getIdFromToken(final String continuationToken);

  String getTokenFromId(final Entity entity);

  class ContinuationTokenException
      extends RuntimeException
  {
    public ContinuationTokenException(final String message, final Exception e) {
      super(message, e);
    }
  }
}
