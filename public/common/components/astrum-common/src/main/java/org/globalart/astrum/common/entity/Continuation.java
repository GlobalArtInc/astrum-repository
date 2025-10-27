
package org.globalart.astrum.common.entity;

import java.util.Collection;

/**
 * Collection of elements with a token that can be used to request the next set of results.
 *
 * @since 3.20
 */
public interface Continuation<E>
    extends Collection<E>, ContinuationAware
{
  // nothing to add
}
