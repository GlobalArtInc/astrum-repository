
package org.globalart.astrum.datastore.api;

import org.globalart.astrum.transaction.Transaction;
import org.globalart.astrum.transaction.TransactionalSession;

/**
 * Represents a session with a {@link DataStore}.
 *
 * @since 3.19
 */
public interface DataSession<T extends Transaction>
    extends TransactionalSession<T>
{
  /**
   * {@link DataAccess} mapping for the given type.
   */
  <D extends DataAccess> D access(Class<D> type);

  /**
   * Registers a hook to run before any changes are committed in this session.
   *
   * @since 3.26
   */
  void preCommit(Runnable hook);

  /**
   * Registers a hook to run after changes have been committed in this session.
   *
   * @since 3.26
   */
  void postCommit(Runnable hook);

  /**
   * Registers a hook to run after changes have been rolled back in this session.
   *
   * @since 3.26
   */
  void onRollback(Runnable hook);

  /**
   * Returns the SQL dialect of the database backing this session.
   *
   * @since 3.26
   */
  String sqlDialect();
}
