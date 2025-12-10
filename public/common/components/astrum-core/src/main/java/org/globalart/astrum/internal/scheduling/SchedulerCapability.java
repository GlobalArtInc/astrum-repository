
package org.globalart.astrum.internal.scheduling;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.common.template.TemplateParameters;
import org.globalart.astrum.scheduling.spi.SchedulerSPI;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Scheduler capability.
 *
 * @since 3.0
 */
@Component(SchedulerCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SchedulerCapability
    extends CapabilitySupport<SchedulerCapabilityConfiguration>
{
  private final SchedulerSPI scheduler;

  private boolean pausedByUs = false;

  @Autowired
  public SchedulerCapability(final SchedulerSPI scheduler) {
    this.scheduler = checkNotNull(scheduler);
  }

  @Override
  protected SchedulerCapabilityConfiguration createConfig(final Map<String, String> properties) throws Exception {
    return new SchedulerCapabilityConfiguration(properties);
  }

  @Override
  protected void onActivate(final SchedulerCapabilityConfiguration config) throws Exception {
    if (pausedByUs) {
      pausedByUs = false;
      scheduler.resume();
    }
  }

  @Override
  protected void onPassivate(final SchedulerCapabilityConfiguration config) throws Exception {
    pausedByUs = true;
    scheduler.pause();
  }

  @Override
  protected String renderDescription() throws Exception {
    return scheduler.renderStatusMessage();
  }

  @Override
  protected String renderStatus() throws Exception {
    return render(SchedulerCapabilityDescriptor.TYPE_ID + "-status.vm", new TemplateParameters()
        .set("detail", scheduler.renderDetailMessage()));
  }
}
