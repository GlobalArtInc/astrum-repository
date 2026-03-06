
package org.globalart.astrum.script.plugin.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.script.Script;
import org.globalart.astrum.script.ScriptCreatedEvent;
import org.globalart.astrum.script.ScriptDeletedEvent;
import org.globalart.astrum.script.ScriptManager;
import org.globalart.astrum.script.ScriptUpdatedEvent;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;
import org.springframework.stereotype.Component;

/**
 * Default {@link ScriptManager}.
 *
 * @since 3.0
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class ScriptManagerImpl
    extends StateGuardLifecycleSupport
    implements ScriptManager
{
  private final EventManager eventManager;

  private final ScriptStore scriptStore;

  private final boolean allowCreation;

  @Autowired
  public ScriptManagerImpl(
      final EventManager eventManager,
      final ScriptStore scriptStore,
      @Value("${astrum.scripts.allowCreation:false}") final boolean allowCreation)
  {
    this.eventManager = checkNotNull(eventManager);
    this.scriptStore = checkNotNull(scriptStore);
    this.allowCreation = checkNotNull(allowCreation);
  }

  @Override
  @Guarded(by = STARTED)
  public Iterable<Script> browse() {
    return ImmutableList.copyOf(scriptStore.list());
  }

  @Override
  @Guarded(by = STARTED)
  public Script get(final String name) {
    return scriptStore.get(name);
  }

  @Override
  @Guarded(by = STARTED)
  public Script create(final String name, final String content, final String type) {
    validateCreationIsAllowed();

    Script script = scriptStore.newScript();
    script.setName(name);
    script.setContent(content);
    script.setType(type);
    scriptStore.create(script);
    eventManager.post(new ScriptCreatedEvent(script));
    return script;
  }

  @Override
  @Guarded(by = STARTED)
  public Script update(final String name, final String content) {
    validateCreationIsAllowed();

    Script script = scriptStore.get(name);
    if (script == null) {
      return null;
    }
    script.setContent(content);
    scriptStore.update(script);
    eventManager.post(new ScriptUpdatedEvent(script));
    return script;
  }

  @Override
  @Guarded(by = STARTED)
  public void delete(final String name) {
    Script script = scriptStore.get(name);
    if (script != null) {
      scriptStore.delete(script);
      eventManager.post(new ScriptDeletedEvent(script));
    }
  }

  @Override
  public boolean isEnabled() {
    return allowCreation;
  }

  private void validateCreationIsAllowed() {
    if (!allowCreation) {
      throw new ScriptingDisabledException("Creating and updating scripts is disable");
    }
  }
}
