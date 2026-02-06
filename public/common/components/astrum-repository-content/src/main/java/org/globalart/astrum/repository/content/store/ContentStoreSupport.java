
package org.globalart.astrum.repository.content.store;

import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Supplier;
import java.util.function.UnaryOperator;

import org.globalart.astrum.common.property.SystemPropertiesHelper;
import org.globalart.astrum.datastore.TransactionalStoreSupport;
import org.globalart.astrum.datastore.api.ContentDataAccess;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.transaction.Transaction;
import org.globalart.astrum.transaction.Transactional;
import org.globalart.astrum.transaction.UnitOfWork;

import static org.globalart.astrum.scheduling.CancelableHelper.checkCancellation;

/**
 * Support class for transactional domain stores backed by a content data store.
 *
 * @since 3.21
 */
public abstract class ContentStoreSupport<T extends ContentDataAccess>
    extends TransactionalStoreSupport<T>
{
  private static final int DELETE_BATCH_SIZE_DEFAULT =
      SystemPropertiesHelper.getInteger("astrum.content.deleteBatchSize", 1000);

  protected ContentStoreSupport(final DataSessionSupplier sessionSupplier, final String contentStoreName) {
    super(sessionSupplier, contentStoreName, ContentStoreSupport.class);
  }

  // alternative constructor that overrides discovery of the DAO class
  protected ContentStoreSupport(
      final DataSessionSupplier sessionSupplier,
      final String contentStoreName,
      final Class<T> daoClass)
  {
    super(sessionSupplier, contentStoreName, ContentStoreSupport.class, daoClass);
  }

  protected DataSession<?> thisSession() {
    return UnitOfWork.currentSession();
  }

  protected T dao() {
    return thisSession().access(daoClass);
  }

  /**
   * Commits any batched changes so far.
   *
   * Also checks to see if the current (potentially long-running) operation has been cancelled.
   */
  protected void commitChangesSoFar() {
    Transaction tx = UnitOfWork.currentTx();
    tx.commit();
    tx.begin();
    checkCancellation();
  }

  protected int deleteBatchSize() {
    return DELETE_BATCH_SIZE_DEFAULT;
  }

  /**
   * Helper to find content in this store before creating it with the given supplier.
   * Automatically retries the operation if another thread creates it just before us.
   */
  @Transactional(retryOn = DuplicateKeyException.class)
  public <D> D getOrCreate(final Supplier<Optional<D>> find, final Supplier<D> create) {
    return find.get().orElseGet(create);
  }

  public <D> D save(
      final Supplier<Optional<D>> find,
      final Supplier<D> create,
      final UnaryOperator<D> update,
      final Consumer<D> postTransaction)
  {
    D result = transactionalSave(find, create, update);
    postTransaction.accept(result);
    return result;
  }

  /**
   * Helper to find content in this store before creating or updating it with the given suppliers.
   *
   * @since 3.30
   */
  @Transactional(retryOn = DuplicateKeyException.class)
  protected <D> D transactionalSave(
      final Supplier<Optional<D>> find,
      final Supplier<D> create,
      final UnaryOperator<D> update)
  {
    return find.get().map(update).orElseGet(create);
  }
}
