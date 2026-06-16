
package org.globalart.astrum.coreui.internal.welcome;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class WelcomeStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String NODE_ID = "astrum.node.id";

  private final NodeAccess nodeAccess;

  private final Boolean featureFlag;

  @Autowired
  public WelcomeStateContributor(
      @Value("${astrum.react.welcome:true}") final Boolean featureFlag,
      final NodeAccess nodeAccess)
  {
    this.nodeAccess = checkNotNull(nodeAccess);
    this.featureFlag = featureFlag;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(
        "astrum.react.welcome", featureFlag,
        NODE_ID, nodeAccess.getId());
  }
}
