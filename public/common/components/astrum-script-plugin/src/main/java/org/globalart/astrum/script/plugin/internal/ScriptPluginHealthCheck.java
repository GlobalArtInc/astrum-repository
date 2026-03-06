
package org.globalart.astrum.script.plugin.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.script.ScriptManager;

import com.codahale.metrics.health.HealthCheck;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Gives an unhealthy response if scripting is enabled in the instance.
 * 
 * @since 3.31
 */
@Component
@Qualifier("Scripting")
public class ScriptPluginHealthCheck
    extends HealthCheck
{
  public static final String SCRIPTING_DISABLED_MESSAGE = "Scripting is disabled.";

  public static final String SCRIPTING_ENABLED_ERROR =
      "Enabling and running scripts is not recommended as this bypasses security checks and can cause your " +
          "Astrum instance to be vulnerable to existing and future attacks. " +
          "We recommend using alternate ways to automate the configuration of this instance where possible.";

  private final ScriptManager scriptManager;

  @Autowired
  public ScriptPluginHealthCheck(final ScriptManager scriptManager) {
    this.scriptManager = scriptManager;
  }

  @Override
  protected Result check() {
    return scriptManager.isEnabled()
        ? Result.unhealthy(SCRIPTING_ENABLED_ERROR)
        : Result.healthy(SCRIPTING_DISABLED_MESSAGE);
  }
}
