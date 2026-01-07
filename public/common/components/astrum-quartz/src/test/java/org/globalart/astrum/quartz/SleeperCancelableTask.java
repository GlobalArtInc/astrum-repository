
package org.globalart.astrum.quartz;

import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Simple sleeper task that is cancelable.
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SleeperCancelableTask
    extends SleeperTask
    implements Cancelable
{
  @Override
  protected void doTheWork() throws Exception {
    super.doTheWork();
    CancelableHelper.checkCancellation();
  }
}
