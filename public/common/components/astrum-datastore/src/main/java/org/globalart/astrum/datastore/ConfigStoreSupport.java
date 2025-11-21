
package org.globalart.astrum.datastore;

import java.util.function.Supplier;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.datastore.api.DataAccess;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.UnitOfWork;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Support class for transactional domain stores backed by the config data store.
 *
 * @since 3.21
 */
public abstract class ConfigStoreSupport<T extends DataAccess>
    extends TransactionalStoreSupport<T>
{
  private EventManager eventManager;

  protected ConfigStoreSupport(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, DEFAULT_DATASTORE_NAME, ConfigStoreSupport.class);
  }

  @VisibleForTesting
  @Autowired
  public void setDependencies(final EventManager eventManager) {
    this.eventManager = checkNotNull(eventManager);
  }

  public void postCommitEvent(final Supplier<?> eventSupplier) {
    thisSession().postCommit(() -> postEvent(eventSupplier));
  }

  private void postEvent(final Supplier<?> eventSupplier) {
    eventManager.post(eventSupplier.get());
  }

  // alternative constructor that overrides discovery of the DAO class
  protected ConfigStoreSupport(final DataSessionSupplier sessionSupplier, final Class<T> daoClass) {
    super(sessionSupplier, DEFAULT_DATASTORE_NAME, ConfigStoreSupport.class, daoClass);
  }

  protected DataSession<?> thisSession() {
    return UnitOfWork.currentSession();
  }

  protected T dao() {
    return thisSession().access(daoClass);
  }
}
