
package org.globalart.astrum.internal.node;

import java.sql.Connection;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.internal.node.datastore.upgrade.UpgradeNodeIdStore;
import org.globalart.astrum.internal.node.upgrade.UpgradeDeploymentIdStore;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class DeploymentIdUpgradeStep_1_23
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeNodeIdStore nodeIdStore;

  private final UpgradeDeploymentIdStore deploymentIdStore;

  @Autowired
  public DeploymentIdUpgradeStep_1_23(
      final UpgradeDeploymentIdStore deploymentIdStore,
      final UpgradeNodeIdStore nodeIdStore)
  {
    this.nodeIdStore = checkNotNull(nodeIdStore);
    this.deploymentIdStore = checkNotNull(deploymentIdStore);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.23");
  }

  @Override
  public void migrate(final Connection connection) {
    Optional<String> deploymentId = deploymentIdStore.get();
    if (deploymentId.isPresent()) {
      return;
    }

    String nodeId = nodeIdStore.getOrCreate();
    // set() throws IllegalStateException on failure, so reaching this point means the write succeeded.
    deploymentIdStore.set(nodeId);
    log.debug("Migration of deployment id was successful.");
  }
}
