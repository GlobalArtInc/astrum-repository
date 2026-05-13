
package org.globalart.astrum.rapture.internal.settings;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.rapture.UiSettingsManager;
import org.globalart.astrum.rapture.settings.RaptureSettings;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.Mockito.verify;

/**
 * Tests for {@link SettingsCapability}.
 */
@RunWith(MockitoJUnitRunner.class)
public class SettingsCapabilityTest
{
  @Mock
  private UiSettingsManager uiSettingsManager;

  @Mock
  private SessionTimeoutManager sessionTimeoutManager;

  private SettingsCapability underTest;

  @Before
  public void setup() {
    underTest = new SettingsCapability(uiSettingsManager, sessionTimeoutManager);
  }

  @Test
  public void testOnActivate_updatesSessionTimeout() throws Exception {
    SettingsCapabilityConfiguration config = createConfig(720);

    underTest.onActivate(config);

    verify(uiSettingsManager).setSettings(config);
    verify(sessionTimeoutManager).updateTimeout(720);
  }

  @Test
  public void testOnActivate_defaultTimeout() throws Exception {
    SettingsCapabilityConfiguration config = createConfig(RaptureSettings.DEFAULT_SESSION_TIMEOUT);

    underTest.onActivate(config);

    verify(uiSettingsManager).setSettings(config);
    verify(sessionTimeoutManager).updateTimeout(RaptureSettings.DEFAULT_SESSION_TIMEOUT);
  }

  @Test
  public void testOnUpdate_updatesSessionTimeout() throws Exception {
    SettingsCapabilityConfiguration config = createConfig(360);

    underTest.onUpdate(config);

    verify(sessionTimeoutManager).updateTimeout(360);
  }

  @Test
  public void testOnUpdate_zeroTimeout() throws Exception {
    SettingsCapabilityConfiguration config = createConfig(0);

    underTest.onUpdate(config);

    verify(sessionTimeoutManager).updateTimeout(0);
  }

  private SettingsCapabilityConfiguration createConfig(int sessionTimeout) {
    Map<String, String> properties = new HashMap<>();
    properties.put(SettingsCapabilityConfiguration.SESSION_TIMEOUT, String.valueOf(sessionTimeout));
    properties.put(SettingsCapabilityConfiguration.TITLE, "Test Title");
    return new SettingsCapabilityConfiguration(properties);
  }
}
