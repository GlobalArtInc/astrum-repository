
package org.globalart.astrum.coreui.internal;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.CLUSTERED_ZERO_DOWNTIME_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.CLUSTERED_ZERO_DOWNTIME_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class ClusteredModeStateContributor
    implements StateContributor
{
  private final boolean clusteredModeEnabled;

  private final boolean zeroDowntimeEnabled;

  @Autowired
  public ClusteredModeStateContributor(
      @Value(DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean clusteredModeEnabled,
      @Value(CLUSTERED_ZERO_DOWNTIME_ENABLED_NAMED_VALUE) final boolean zeroDowntimeEnabled)
  {
    this.clusteredModeEnabled = clusteredModeEnabled;
    this.zeroDowntimeEnabled = zeroDowntimeEnabled;
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(
        DATASTORE_CLUSTERED_ENABLED, clusteredModeEnabled,
        CLUSTERED_ZERO_DOWNTIME_ENABLED, zeroDowntimeEnabled);
  }
}
