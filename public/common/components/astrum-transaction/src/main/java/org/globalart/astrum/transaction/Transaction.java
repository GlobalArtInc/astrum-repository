
package org.globalart.astrum.transaction;

/**
 * Represents a restartable transaction that can allow zero or more retries.
 *
 * @since 3.0
 */
public interface Transaction
{
  void begin();

  void commit();

  void rollback();

  /**
   * Invoked at the end of the transaction
   */
  default void end() {

  };

  boolean isActive();

  /**
   * @param cause The cause of this retry request
   *
   * @return {@code true} if we're allowed to try the work again, otherwise {@code false}<br>
   *         (avoid always returning {@code true} unless you want infinite retries)
   *
   * @throws RuntimeException may be thrown to implicitly deny the retry
   */
  boolean allowRetry(Exception cause);

  /**
   * Notifies this transaction if it captures another {@link TransactionalStore} during a nested transaction.
   *
   * @since 3.20
   */
  default void capture(TransactionalStore<?> store) {
    // do nothing by default
  }

  /**
   * @see Transactional#reason()
   * @since 3.20
   */
  void reason(String reason);

  /**
   * @see Transactional#reason()
   * @since 3.20
   */
  String reason();
}
