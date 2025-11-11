
package org.globalart.astrum.internal.event;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.property.SystemPropertiesHelper;
import org.globalart.astrum.jmx.reflect.ManagedAttribute;
import org.globalart.astrum.jmx.reflect.ManagedObject;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A simple "debug" helper component, that dumps out events to log.
 *
 * Usable for debugging or problem solving, not for production use!
 *
 * It will register itself to listen for events only when enabled,
 * otherwise it will not spend any CPU cycles being dormant.
 *
 * It can be enabled via System property or JMX.
 *
 * @author cstamas
 * @since 2.1
 */
@Component
@ManagedObject
public class DebugEventInspector
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final boolean ENABLED_DEFAULT = SystemPropertiesHelper.getBoolean(
      DebugEventInspector.class.getName() + ".enabled", false);

  private volatile boolean enabled;

  private final EventManager eventManager;

  @Autowired
  public DebugEventInspector(final EventManager eventManager) {
    this.eventManager = checkNotNull(eventManager);
    setEnabled(ENABLED_DEFAULT);
  }

  @ManagedAttribute
  public boolean isEnabled() {
    return enabled;
  }

  @ManagedAttribute
  public void setEnabled(final boolean enabled) {
    try {
      if (enabled && !this.enabled) {
        eventManager.register(this);
      }
      else if (!enabled && this.enabled) {
        eventManager.unregister(this);
      }
    }
    finally {
      this.enabled = enabled;
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void accept(final Object event) {
    log.info("{}", event);
  }
}
