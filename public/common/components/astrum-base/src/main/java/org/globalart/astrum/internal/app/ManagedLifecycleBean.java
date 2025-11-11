
package org.globalart.astrum.internal.app;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.globalart.astrum.common.app.ManagedLifecycleManager;
import org.globalart.astrum.jmx.reflect.ManagedAttribute;
import org.globalart.astrum.jmx.reflect.ManagedObject;
import org.globalart.astrum.jmx.reflect.ManagedOperation;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * JMX controller to manage the Astrum application lifecycle.
 *
 * @since 3.16
 */
@Component
@ManagedObject
public class ManagedLifecycleBean
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ManagedLifecycleManager lifecycleManager;

  @Autowired
  public ManagedLifecycleBean(final ManagedLifecycleManager lifecycleManager) {
    this.lifecycleManager = checkNotNull(lifecycleManager);
  }

  @ManagedAttribute
  public String getPhase() {
    return lifecycleManager.getCurrentPhase().name();
  }

  @ManagedAttribute
  public void setPhase(final String phase) {
    try {
      lifecycleManager.to(Phase.valueOf(phase));
    }
    catch (Exception e) {
      log.warn("Problem moving to phase {}", phase, e);
      throw new RuntimeException("Problem moving to phase " + phase + ": " + e);
    }
  }

  @ManagedOperation
  public void bounce(final String phase) {
    try {
      lifecycleManager.bounce(Phase.valueOf(phase));
    }
    catch (Exception e) {
      log.warn("Problem bouncing phase {}", phase, e);
      throw new RuntimeException("Problem bouncing phase " + phase + ": " + e);
    }
  }
}
