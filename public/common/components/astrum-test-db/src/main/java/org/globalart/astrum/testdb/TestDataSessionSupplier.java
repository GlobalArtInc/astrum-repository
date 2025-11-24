
package org.globalart.astrum.testdb;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.stream.Stream;

import javax.sql.DataSource;

import org.globalart.astrum.datastore.api.DataAccess;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.mybatis.MyBatisDataStore;
import org.globalart.astrum.transaction.TransactionIsolation;

import org.assertj.db.type.Table;

public class TestDataSessionSupplier
    implements DataSessionSupplier
{
  private final String storeName;

  private final MyBatisDataStore store;

  TestDataSessionSupplier(final String storeName, final MyBatisDataStore store) {
    this.storeName = storeName;
    this.store = store;
  }

  @Override
  public DataSession<?> openSession(final String storeName) {
    checkName(storeName);
    return store.openSession();
  }

  @Override
  public DataSession<?> openSerializableTransactionSession(final String storeName) {
    checkName(storeName);
    return store.openSession(TransactionIsolation.SERIALIZABLE);
  }

  @Override
  public Connection openConnection(final String storeName) throws SQLException {
    checkName(storeName);
    return store.openConnection();
  }

  @Override
  public Optional<DataSource> getDataSource(final String storeName) {
    checkName(storeName);
    return Optional.of(store.getDataSource());
  }

  public DataSession<?> openSession() {
    return store.openSession();
  }

  public DataSession<?> openSerializableTransactionSession() {
    return store.openSession(TransactionIsolation.SERIALIZABLE);
  }

  public Connection openConnection() throws SQLException {
    return store.openConnection();
  }

  public Optional<DataSource> getDataSource() {
    return Optional.of(store.getDataSource());
  }

  /**
   * Invokes the provided callback with the specified DAO returning the callback provided value. Transaction will be
   * committed.
   */
  public <D extends DataAccess, R> R withDAO(final Class<D> clazz, final Function<D, R> callback) {
    try (DataSession<?> session = store.openSession()) {
      R result = callback.apply(session.access(clazz));
      session.getTransaction().commit();
      return result;
    }
  }

  /**
   * Invokes the provided callback with the specified DAO. Transaction will be committed.
   */
  public <D extends DataAccess> void callDAO(final Class<D> clazz, final Consumer<D> callback) {
    try (DataSession<?> session = store.openSession()) {
      callback.accept(session.access(clazz));
      session.getTransaction().commit();
    }
  }

  public Table table(final String name) {
    return new Table(getDataSource().get(), name);
  }

  private void checkName(final String suppliedName) {
    if (!storeName.equals(suppliedName)) {
      throw new IllegalArgumentException(
          "Requested store %s does not match created store %s".formatted(suppliedName, storeName));
    }
  }

  @SafeVarargs
  public final void register(final Class<? extends DataAccess>... daoClazz) {
    Stream.of(daoClazz).forEach(store::register);
  }

  public DataStore<?> getDataStore(final String storeName) {
    checkName(storeName);
    return store;
  }
}
