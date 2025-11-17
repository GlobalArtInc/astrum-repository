
package org.globalart.astrum.internal.node;

import java.sql.Connection;

import org.globalart.astrum.internal.node.datastore.NodeIdDAO;
import org.globalart.astrum.internal.node.datastore.upgrade.UpgradeNodeIdStore;
import org.globalart.astrum.internal.node.upgrade.UpgradeDeploymentIdStore;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Real-database tests for {@link DeploymentIdUpgradeStep_1_23} using the direct-SQL Upgrade stores.
 */
class DeploymentIdUpgradeStep_1_23Test
{
  @DataSessionConfiguration(daos = {NodeIdDAO.class, DeploymentIdDAO.class})
  TestDataSessionSupplier dataSessionSupplier;

  private UpgradeNodeIdStore nodeIdStore() {
    return new UpgradeNodeIdStore(dataSessionSupplier);
  }

  private UpgradeDeploymentIdStore deploymentIdStore() {
    return new UpgradeDeploymentIdStore(dataSessionSupplier);
  }

  private DeploymentIdUpgradeStep_1_23 underTest() {
    return new DeploymentIdUpgradeStep_1_23(deploymentIdStore(), nodeIdStore());
  }

  @DatabaseTest
  void migrate_whenNoDeploymentId_setsItFromNewNodeId() throws Exception {
    try (Connection conn = dataSessionSupplier.openConnection()) {
      underTest().migrate(conn);
    }

    String nodeId = nodeIdStore().get().orElseThrow();
    assertThat(deploymentIdStore().get()).contains(nodeId);
  }

  @DatabaseTest
  void migrate_whenDeploymentIdExists_isNoOp() throws Exception {
    deploymentIdStore().set("existing-deployment");

    try (Connection conn = dataSessionSupplier.openConnection()) {
      underTest().migrate(conn);
    }

    assertThat(deploymentIdStore().get()).contains("existing-deployment");
    // node id was never created because the migration short-circuited
    assertThat(nodeIdStore().get()).isEmpty();
  }
}
