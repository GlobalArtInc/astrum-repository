
package org.globalart.astrum.coreui.internal.repository;

import java.util.Map;

import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import jakarta.inject.Inject;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.FeatureFlags.RAW_QUERYPARAMS_FORWARDING_ENABLED_NAMED_VALUE;

/**
 * Exposes the {@code rawQueryParamsForwardingEnabled} flag to the UI via Rapture state,
 * controlling visibility of the query parameter forwarding configuration panel
 * in Raw proxy repository forms.
 */
@Component
public class RawQueryParamsForwardingStateContributor
    implements StateContributor
{
  private final boolean isRawQueryParamsForwardingEnabled;

  @Inject
  public RawQueryParamsForwardingStateContributor(
      @Value(RAW_QUERYPARAMS_FORWARDING_ENABLED_NAMED_VALUE) final boolean isRawQueryParamsForwardingEnabled)
  {
    this.isRawQueryParamsForwardingEnabled = isRawQueryParamsForwardingEnabled;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of("rawQueryParamsForwardingEnabled", isRawQueryParamsForwardingEnabled);
  }
}
