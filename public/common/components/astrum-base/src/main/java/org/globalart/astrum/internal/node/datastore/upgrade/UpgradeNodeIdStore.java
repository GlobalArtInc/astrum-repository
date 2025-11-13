
package org.globalart.astrum.internal.node.datastore.upgrade;

import java.util.Optional;
import java.util.UUID;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.internal.node.NodeIdEncoding;
import org.globalart.astrum.internal.node.upgrade.SingleRowStringUpgradeStore;

import com.google.common.hash.Hashing;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;

/**
 * UPGRADE-phase-safe equivalent of {@code NodeIdStore} that reads/writes the {@code node_id} table
 * directly via SQL, avoiding the {@code ConfigStoreSupport} -> {@code EventManager} (EVENTS phase)
 * dependency.
 */
@Component
@ManagedLifecycle(phase = UPGRADE)
public class UpgradeNodeIdStore
    extends SingleRowStringUpgradeStore
{
  @Autowired
  public UpgradeNodeIdStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, "node_id", "node_id");
  }

  public Optional<String> get() {
    return read();
  }

  public void set(final String nodeId) {
    write(nodeId);
  }

  public String getOrCreate() {
    return get().orElseGet(() -> {
      String newNodeId = generateNodeId();
      set(newNodeId);
      return newNodeId;
    });
  }

  // SHA-1 (and the platform-default charset) are preserved deliberately: this mirrors the production
  // NodeIdStore algorithm so migrated node IDs match the values legacy installs already generated.
  // Changing the hash or encoding here would produce different IDs and break that compatibility.
  @SuppressWarnings("deprecation")
  private String generateNodeId() {
    UUID cn = UUID.randomUUID();
    return NodeIdEncoding.nodeIdForSha1(Hashing.sha1().hashBytes(cn.toString().getBytes()).toString());
  }
}
