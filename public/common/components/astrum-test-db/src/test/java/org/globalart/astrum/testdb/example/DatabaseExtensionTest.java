
package org.globalart.astrum.testdb.example;

import java.sql.SQLException;

import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import com.google.common.collect.ImmutableMap;

import static org.assertj.db.api.Assertions.assertThat;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.notNullValue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

class DatabaseExtensionTest
{
  @DataSessionConfiguration(daos = TestItemDAO.class)
  TestDataSessionSupplier supplier;

  @DatabaseTest
  void testGetDataSource() {
    assertThat(supplier.getDataSource().orElse(null), notNullValue());
    assertThat(supplier.getDataSource(DEFAULT_DATASTORE_NAME).orElse(null), notNullValue());
  }

  @DatabaseTest
  void testOpenConnection() throws SQLException {
    assertThat(supplier.openConnection(), notNullValue());
    assertThat(supplier.openConnection(DEFAULT_DATASTORE_NAME), notNullValue());
  }

  @DatabaseTest
  void testOpenSession() {
    assertThat(supplier.openSession(), notNullValue());
    assertThat(supplier.openSession(DEFAULT_DATASTORE_NAME), notNullValue());
  }

  @DatabaseTest
  void testOpenSerializableTransactionSession() {
    assertThat(supplier.openSerializableTransactionSession(), notNullValue());
    assertThat(supplier.openSerializableTransactionSession(DEFAULT_DATASTORE_NAME), notNullValue());
  }

  @DatabaseTest
  void testStoresCreated() {
    assertThat(supplier.table("test_item")).exists();
  }

  @DatabaseTest
  void testCallDAO() {
    TestItem item = new TestItem();
    item.setVersion(1);
    item.setEnabled(true);
    item.setNotes("test-entity");
    item.setProperties(ImmutableMap.of("sample", "data"));

    supplier.callDAO(TestItemDAO.class, dao -> dao.create(item));
    assertThat(supplier.table("test_item")).hasNumberOfRows(1);
  }

  @DatabaseTest
  void testWithDAO() {
    Iterable<TestItem> iter = supplier.withDAO(TestItemDAO.class, dao -> dao.browse());
    assertThat(iter, notNullValue());
  }
}
