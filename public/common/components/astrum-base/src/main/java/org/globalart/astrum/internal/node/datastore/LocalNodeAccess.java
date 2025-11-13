
package org.globalart.astrum.internal.node.datastore;

import java.util.Collections;
import java.util.Map;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.node.NodeAccessSupport;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.node.datastore.NodeIdStore;

import com.google.common.collect.ImmutableMap;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Local {@link NodeAccess}.
 *
 * @since 3.0
 */
@Component
@Qualifier("local")
public class LocalNodeAccess
    extends NodeAccessSupport
{
  private final NodeIdStore nodeIdStore;

  private String id;

  private Map<String, String> memberAliases = Collections.emptyMap();

  @Autowired
  public LocalNodeAccess(final NodeIdStore nodeIdStore) {
    this.nodeIdStore = checkNotNull(nodeIdStore);
  }

  @Override
  protected void doStart() throws Exception {
    this.id = nodeIdStore.getOrCreate();

    log.info("ID: {}", id);

    memberAliases = ImmutableMap.of(id, id);
  }

  @Override
  protected void doStop() throws Exception {
    id = null;
  }

  @Override
  @Guarded(by = STARTED)
  public String getId() {
    return id;
  }

  @Override
  @Guarded(by = STARTED)
  public String getClusterId() {
    return getId();
  }

  @Override
  public boolean isClustered() {
    return false;
  }

  @Override
  public boolean isOldestNode() {
    return true;
  }

  @Override
  public Set<String> getMemberIds() {
    return memberAliases.keySet();
  }

  @Override
  public Map<String, String> getMemberAliases() {
    return memberAliases;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id='" + id + '\'' +
        '}';
  }
}
