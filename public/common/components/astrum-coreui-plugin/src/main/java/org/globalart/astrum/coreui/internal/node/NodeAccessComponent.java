
package org.globalart.astrum.coreui.internal.node;

import java.util.List;
import java.util.Map.Entry;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.extdirect.DirectComponentSupport;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import org.springframework.stereotype.Component;

/**
 * NodeAccessComponent {@link DirectComponentSupport}.
 */
@Component
@DirectAction(action = "node_NodeAccess")
public class NodeAccessComponent
    extends DirectComponentSupport
{
  private final NodeAccess nodeAccess;

  @Autowired
  public NodeAccessComponent(final NodeAccess nodeAccess) {
    this.nodeAccess = checkNotNull(nodeAccess);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<NodeInfoXO> nodes() {
    return nodeAccess.getMemberAliases().entrySet().stream().map(this::asNodeInfoXO).collect(toList());
  }

  private NodeInfoXO asNodeInfoXO(final Entry<String, String> entry) {
    NodeInfoXO nodeInfoXO = new NodeInfoXO();
    nodeInfoXO.setName(entry.getKey());
    nodeInfoXO.setLocal(entry.getKey().equals(nodeAccess.getId()));
    nodeInfoXO.setDisplayName(entry.getValue());
    return nodeInfoXO;
  }
}
