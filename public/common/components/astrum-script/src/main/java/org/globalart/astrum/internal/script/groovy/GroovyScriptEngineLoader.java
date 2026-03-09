
package org.globalart.astrum.internal.script.groovy;

import org.springframework.beans.factory.annotation.Autowired;
import javax.script.ScriptEngineManager;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import org.springframework.stereotype.Component;

/**
 * Groovy script engine loader. Is used to run groovy scripts.
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class GroovyScriptEngineLoader
    extends StateGuardLifecycleSupport
{
  private final ApplicationDirectories applicationDirectories;

  private final ScriptEngineManager scriptEngineManager;

  @Autowired
  public GroovyScriptEngineLoader(
      final ApplicationDirectories applicationDirectories,
      final ScriptEngineManager scriptEngineManager)
  {
    this.applicationDirectories = checkNotNull(applicationDirectories);
    this.scriptEngineManager = checkNotNull(scriptEngineManager);
  }

  @Override
  protected void doStart() throws Exception {
    GroovyScriptEngineFactory groovyEngineFactory =
        new GroovyScriptEngineFactory(applicationDirectories);
    log.debug("Registering engine-factory: {}", groovyEngineFactory);
    groovyEngineFactory.getNames().forEach(name -> scriptEngineManager.registerEngineName(name, groovyEngineFactory));
  }
}
