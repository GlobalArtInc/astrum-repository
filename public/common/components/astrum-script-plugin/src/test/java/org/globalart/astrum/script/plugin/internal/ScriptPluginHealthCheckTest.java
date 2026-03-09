
package org.globalart.astrum.script.plugin.internal;

import org.globalart.astrum.script.ScriptManager;

import com.codahale.metrics.health.HealthCheck.Result;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.junit.Assert.assertTrue;
import static org.junit.Assert.assertFalse;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ScriptPluginHealthCheckTest
{
  @Mock
  ScriptManager scriptManager;

  private ScriptPluginHealthCheck underTest;

  @Before
  public void setup() {
    underTest = new ScriptPluginHealthCheck(scriptManager);
  }

  @Test
  public void checkScriptManagerDisabled() {
    when(scriptManager.isEnabled()).thenReturn(false);
    Result result = underTest.check();
    assertTrue(result.isHealthy());
  }

  @Test
  public void checkScriptManagerEnabled() {
    when(scriptManager.isEnabled()).thenReturn(true);
    Result result = underTest.check();
    assertFalse(result.isHealthy());
  }
}
