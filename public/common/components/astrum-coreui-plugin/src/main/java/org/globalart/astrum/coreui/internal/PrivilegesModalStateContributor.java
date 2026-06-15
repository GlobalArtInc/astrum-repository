
package org.globalart.astrum.coreui.internal;

import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES_MODAL_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES_MODAL_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class PrivilegesModalStateContributor
    implements StateContributor
{
  private final boolean isPrivilegesModalEnabled;

  @Autowired
  public PrivilegesModalStateContributor(
      @Value(REACT_PRIVILEGES_MODAL_NAMED_VALUE) final boolean isPrivilegesModalEnabled)
  {
    this.isPrivilegesModalEnabled = isPrivilegesModalEnabled;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(REACT_PRIVILEGES_MODAL_ENABLED, isPrivilegesModalEnabled);
  }
}
