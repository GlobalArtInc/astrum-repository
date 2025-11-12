
package org.globalart.astrum.internal.node;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.node.DeploymentAccess;
import org.globalart.astrum.common.node.NodeAccess;

import static com.google.common.base.Preconditions.checkNotNull;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

/**
 * Simple {@link DeploymentAccess} that just delegates to {@link NodeAccess} and doesn't allow aliases.
 *
 * @since 3.21
 */
@Component
@Primary
public class SimpleDeploymentAccess
    implements DeploymentAccess
{
  private final DeploymentIdStore deploymentIdStore;

  @Autowired
  public SimpleDeploymentAccess(final DeploymentIdStore deploymentIdStore) {
    this.deploymentIdStore = checkNotNull(deploymentIdStore);
  }

  @Override
  public String getId() {
    return deploymentIdStore.get()
        .orElseThrow(() -> new RuntimeException("Deployment id is absent."));
  }

  @Override
  public String getAlias() {
    return null; // alias is never set
  }

  @Override
  public void setAlias(final String newAlias) {
    throw new UnsupportedOperationException();
  }

  @Override
  public void start() {
    // no-op
  }

  @Override
  public void stop() {
    // no-op
  }
}
