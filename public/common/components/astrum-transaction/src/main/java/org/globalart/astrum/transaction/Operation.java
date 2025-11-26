
package org.globalart.astrum.transaction;

import java.util.concurrent.Callable;

/**
 * Like {@link Callable} but with a more specific throws clause.
 * 
 * @since 3.0
 */
@FunctionalInterface
public interface Operation<T, E extends Exception>
{
  T call() throws E;
}
