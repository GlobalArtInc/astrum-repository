
package org.globalart.astrum.coreui.internal.previewui;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_SETTINGS_ENABLED_NAMED_VALUE;

/**
 * Exposes {@code astrum.previewui.enabled} to the frontend via {@code AS.State}.
 * When false, the Preview UI Settings page is hidden from the legacy admin UI. Defaults to true.
 */
@Component
public class PreviewUiEnabledStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, Object> state;

  @Autowired
  public PreviewUiEnabledStateContributor(
      @Value(PREVIEW_UI_SETTINGS_ENABLED_NAMED_VALUE) final Boolean featureFlag)
  {
    state = ImmutableMap.of("astrum.previewui.enabled", featureFlag);
  }

  @Override
  public Map<String, Object> getState() {
    return state;
  }
}
