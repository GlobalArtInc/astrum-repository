
package org.globalart.astrum.internal.node;

import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class DeploymentIdStoreImpl
    extends ConfigStoreSupport<DeploymentIdDAO>
    implements DeploymentIdStore
{
  private boolean initialized;

  private Optional<String> deploymentId;

  @Autowired
  public DeploymentIdStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  @Override
  public Optional<String> get() {
    if (!initialized) {
      deploymentId = dao().get();

      if (deploymentId.isPresent()) {
        initialized = true;
      }
    }

    return deploymentId;
  }

  @Transactional
  @Override
  public void set(final String deploymentId) {
    checkNotNull(deploymentId);
    dao().set(deploymentId);

    this.deploymentId = Optional.of(deploymentId);
    this.initialized = true;
  }
}
