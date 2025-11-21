
package org.globalart.astrum.datastore;

import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.TransactionIsolation;
import org.globalart.astrum.transaction.TransactionalStore;

import com.google.common.collect.Iterables;
import org.apache.commons.lang3.reflect.TypeUtils;

import static com.google.common.base.Preconditions.checkNotNull;

public abstract class TransactionalStoreSupport<DAO>
    extends StateGuardLifecycleSupport
    implements TransactionalStore<DataSession<?>>
{
  protected final Class<DAO> daoClass;

  protected final DataSessionSupplier sessionSupplier;

  private final String storeName;

  @SuppressWarnings({"unchecked", "rawtypes"})
  protected TransactionalStoreSupport(
      final DataSessionSupplier sessionSupplier,
      final String storeName,
      final Class<? extends TransactionalStoreSupport> storeParentClass)
  {
    this.sessionSupplier = checkNotNull(sessionSupplier);
    this.storeName = checkNotNull(storeName);
    this.daoClass = (Class) Iterables.getOnlyElement(TypeUtils.getTypeArguments(getClass(), storeParentClass).values());
  }

  @SuppressWarnings({"rawtypes"})
  protected TransactionalStoreSupport(
      final DataSessionSupplier sessionSupplier,
      final String storeName,
      final Class<? extends TransactionalStoreSupport> storeParentClass,
      final Class<DAO> daoClass)
  {
    this.sessionSupplier = checkNotNull(sessionSupplier);
    this.storeName = checkNotNull(storeName);
    this.daoClass = daoClass;
  }

  @Override
  public DataSession<?> openSession() {
    return sessionSupplier.openSession(storeName);
  }

  @Override
  public DataSession<?> openSession(final TransactionIsolation isolationLevel) {
    switch (isolationLevel) {
      case SERIALIZABLE:
        log.debug("Opening session with serializable transaction isolation");
        return sessionSupplier.openSerializableTransactionSession(storeName);
      default:
        return sessionSupplier.openSession(storeName);
    }
  }

  public Class<DAO> getDaoClass() {
    return daoClass;
  }
}
