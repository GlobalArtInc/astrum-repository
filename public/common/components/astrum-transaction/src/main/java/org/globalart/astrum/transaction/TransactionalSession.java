
package org.globalart.astrum.transaction;

/**
 * Represents an open session with a {@link TransactionalStore}.
 *
 * @since 3.19
 */
public interface TransactionalSession<T extends Transaction>
    extends AutoCloseable
{
  /**
   * Gets the {@link Transaction} control for this session.
   */
  T getTransaction();

  /**
   * Closes this session.
   */
  @Override
  void close();
}
