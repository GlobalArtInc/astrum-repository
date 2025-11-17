
package org.globalart.astrum.internal.node.upgrade;

import org.globalart.astrum.internal.node.DeploymentIdDAO;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Real-database tests for {@link UpgradeDeploymentIdStore} against the {@code deployment_id} table.
 */
class UpgradeDeploymentIdStoreTest
{
  @DataSessionConfiguration(daos = {DeploymentIdDAO.class})
  TestDataSessionSupplier dataSessionSupplier;

  private UpgradeDeploymentIdStore store() {
    return new UpgradeDeploymentIdStore(dataSessionSupplier);
  }

  @DatabaseTest
  void get_whenEmptyTable_returnsEmpty() {
    assertThat(store().get()).isEmpty();
  }

  @DatabaseTest
  void set_whenEmpty_persistsValue() {
    UpgradeDeploymentIdStore store = store();
    store.set("deployment-1");
    assertThat(store.get()).contains("deployment-1");
  }

  @DatabaseTest
  void set_whenValueExists_overwrites() {
    UpgradeDeploymentIdStore store = store();
    store.set("first");
    store.set("second");
    assertThat(store.get()).contains("second");
  }
}
