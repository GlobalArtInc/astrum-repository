
package org.globalart.astrum.internal.node.datastore;

import java.util.Optional;
import java.util.UUID;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.internal.node.NodeIdEncoding;
import org.globalart.astrum.node.datastore.NodeIdStore;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.hash.Hashing;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.37
 */
@Component
@Qualifier("mybatis")
public class NodeIdStoreImpl
    extends ConfigStoreSupport<NodeIdDAO>
    implements NodeIdStore
{
  @Autowired
  public NodeIdStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  /**
   * Remove the currently persisted node id, this will not change the {@link NodeAccess}.
   */
  @Transactional
  @Override
  public void clear() {
    dao().clear();
  }

  /**
   * Retrieve the current node id if it exists.
   *
   * @return the node id
   */
  @Transactional
  @Override
  public Optional<String> get() {
    return dao().get();
  }

  /**
   * Set the current node id, this will not update the {@link NodeAccess}
   *
   * @param nodeId
   */
  @Transactional
  @Override
  public void set(final String nodeId) {
    dao().set(nodeId);
  }

  @Transactional(retryOn = DuplicateKeyException.class)
  @Override
  public String getOrCreate() {
    return get()
        .orElseGet(() -> {
          String newNodeId = generateNodeId();
          dao().create(newNodeId);
          return newNodeId;
        });
  }

  private String generateNodeId() {
    log.debug("Generating nodeId");

    // Generate something unique
    UUID cn = UUID.randomUUID();

    // Hash it to match old certificate style
    @SuppressWarnings("deprecation")
    String newNodeId = NodeIdEncoding.nodeIdForSha1(Hashing.sha1().hashBytes(cn.toString().getBytes()).toString());

    return newNodeId;
  }
}
