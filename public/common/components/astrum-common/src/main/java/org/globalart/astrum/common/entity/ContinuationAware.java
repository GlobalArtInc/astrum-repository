
package org.globalart.astrum.common.entity;

/**
 * Something that's aware of what the next {@link Continuation} token will be.
 *
 * @since 3.20
 */
public interface ContinuationAware
{
  /**
   * Use this token when requesting the next set of results.
   */
  String nextContinuationToken();
}
