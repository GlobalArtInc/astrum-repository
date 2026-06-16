
package org.globalart.astrum.coreui.internal.privileges;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class PrivilegesStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, Object> state;

  @Autowired
  public PrivilegesStateContributor(
      @Value(REACT_PRIVILEGES_NAMED_VALUE) final Boolean featureFlag)
  {
    state = ImmutableMap.of(REACT_PRIVILEGES, featureFlag);
  }

  @Override
  public Map<String, Object> getState() {
    return state;
  }
}
