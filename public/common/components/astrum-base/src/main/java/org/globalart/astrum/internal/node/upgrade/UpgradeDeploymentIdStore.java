
package org.globalart.astrum.internal.node.upgrade;

import java.util.Optional;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.datastore.api.DataSessionSupplier;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;

/**
 * UPGRADE-phase-safe equivalent of {@code DeploymentIdStore} that reads/writes the {@code deployment_id}
 * table directly via SQL, avoiding the {@code ConfigStoreSupport} -> {@code EventManager} (EVENTS phase)
 * dependency.
 */
@Component
@ManagedLifecycle(phase = UPGRADE)
public class UpgradeDeploymentIdStore
    extends SingleRowStringUpgradeStore
{
  @Autowired
  public UpgradeDeploymentIdStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, "deployment_id", "deployment_id");
  }

  public Optional<String> get() {
    return read();
  }

  public void set(final String deploymentId) {
    write(deploymentId);
  }
}
