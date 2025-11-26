
package org.globalart.astrum.transaction;

/**
 * Like {@link Runnable} but with a specific throws clause.
 *
 * @since 3.2
 */
@FunctionalInterface
public interface VoidOperation<E extends Exception>
{
  void run() throws E;
}
