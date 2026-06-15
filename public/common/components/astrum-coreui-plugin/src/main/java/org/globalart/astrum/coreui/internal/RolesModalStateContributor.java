
package org.globalart.astrum.coreui.internal;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.REACT_ROLES_MODAL_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_ROLES_MODAL_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class RolesModalStateContributor
    implements StateContributor
{
  private final boolean isRolesModalEnabled;

  @Autowired
  public RolesModalStateContributor(
      @Value(REACT_ROLES_MODAL_NAMED_VALUE) final boolean isRolesModalEnabled)
  {
    this.isRolesModalEnabled = isRolesModalEnabled;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(REACT_ROLES_MODAL_ENABLED, isRolesModalEnabled);
  }
}
