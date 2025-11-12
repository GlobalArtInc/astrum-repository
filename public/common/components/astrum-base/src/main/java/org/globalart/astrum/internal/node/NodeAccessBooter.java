
package org.globalart.astrum.internal.node;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.node.NodeAccess;

import org.springframework.beans.factory.ObjectProvider;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Objects.isNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.STORAGE;

/**
 * {@link NodeAccess} booter.
 *
 * @since 3.24
 */
@Component
@ManagedLifecycle(phase = STORAGE)
@Order(Ordered.HIGHEST_PRECEDENCE + 1) // make sure this starts first
public class NodeAccessBooter
    implements Lifecycle
{
  private final ObjectProvider<NodeIdInitializer> nodeIdInitializer;

  private final NodeAccess nodeAccess;

  @Autowired
  public NodeAccessBooter(final ObjectProvider<NodeIdInitializer> nodeIdInitializer, final NodeAccess nodeAccess) {
    this.nodeIdInitializer = checkNotNull(nodeIdInitializer);
    this.nodeAccess = checkNotNull(nodeAccess);
  }

  @Override
  public void start() throws Exception {
    NodeIdInitializer nodeIdInitializer = this.nodeIdInitializer.getIfAvailable();
    if (!isNull(nodeIdInitializer)) {
      nodeIdInitializer.initialize();
    }
    nodeAccess.start();
  }

  @Override
  public void stop() throws Exception {
    nodeAccess.stop();
  }
}
