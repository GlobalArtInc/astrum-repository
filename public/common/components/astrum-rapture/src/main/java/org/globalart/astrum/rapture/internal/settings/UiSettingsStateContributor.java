
package org.globalart.astrum.rapture.internal.settings;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.rapture.UiSettingsManager;

import com.google.common.collect.ImmutableMap;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Contributes {@code uiSettings} state.
 *
 * @since 3.0
 */
@Component
public class UiSettingsStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String STATE_ID = "uiSettings";

  private final UiSettingsManager rapture;

  @Autowired
  public UiSettingsStateContributor(final UiSettingsManager rapture) {
    this.rapture = checkNotNull(rapture);
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(STATE_ID, calculateSettings());
  }

  private Object calculateSettings() {
    return rapture.getSettings();
  }
}
