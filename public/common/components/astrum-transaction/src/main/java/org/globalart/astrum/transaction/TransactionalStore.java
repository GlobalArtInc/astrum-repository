
package org.globalart.astrum.transaction;

/**
 * Represents some kind of storage with transactional behaviour.
 *
 * @since 3.19
 */
public interface TransactionalStore<S extends TransactionalSession<?>>
{
  /**
   * Opens a new {@link TransactionalSession}.
   */
  S openSession();

  /**
   * Open a new {@link TransactionalSession} with the provided {@link TransactionIsolation} level. May not be supported
   * by all types of stores.
   *
   * @param isolationLevel the isolation level to use for the transaction
   */
  default S openSession(final TransactionIsolation isolationLevel) {
    switch (isolationLevel) {
      case STANDARD:
        return openSession();
      default:
        throw new UnsupportedOperationException();
    }
  }
}
