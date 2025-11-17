
package org.globalart.astrum.internal.node.datastore.upgrade;

import org.globalart.astrum.internal.node.datastore.NodeIdDAO;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Real-database tests for {@link UpgradeNodeIdStore} against the {@code node_id} table.
 */
class UpgradeNodeIdStoreTest
{
  @DataSessionConfiguration(daos = {NodeIdDAO.class})
  TestDataSessionSupplier dataSessionSupplier;

  private UpgradeNodeIdStore store() {
    return new UpgradeNodeIdStore(dataSessionSupplier);
  }

  @DatabaseTest
  void get_whenEmptyTable_returnsEmpty() {
    assertThat(store().get()).isEmpty();
  }

  @DatabaseTest
  void set_whenEmpty_persistsValue() {
    UpgradeNodeIdStore store = store();
    store.set("foo");
    assertThat(store.get()).contains("foo");
  }

  @DatabaseTest
  void set_whenValueExists_overwrites() {
    UpgradeNodeIdStore store = store();
    store.set("a");
    store.set("b");
    assertThat(store.get()).contains("b");
  }

  @DatabaseTest
  void getOrCreate_whenEmpty_generatesAndPersists() {
    UpgradeNodeIdStore store = store();
    String created = store.getOrCreate();
    assertThat(created).isNotBlank();
    assertThat(store.get()).contains(created);
  }

  @DatabaseTest
  void getOrCreate_whenPresent_returnsExisting() {
    UpgradeNodeIdStore store = store();
    store.set("existing-node");
    assertThat(store.getOrCreate()).isEqualTo("existing-node");
  }
}
