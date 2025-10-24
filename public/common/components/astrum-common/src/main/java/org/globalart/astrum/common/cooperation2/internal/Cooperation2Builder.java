
package org.globalart.astrum.common.cooperation2.internal;

import java.util.Optional;

import org.globalart.astrum.common.cooperation2.Cooperation2.Builder;
import org.globalart.astrum.common.cooperation2.IOCall;
import org.globalart.astrum.common.cooperation2.IOCheck;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Abstract implementation of {@link Builder}
 */
public abstract class Cooperation2Builder<RET>
    implements Builder<RET>
{
  protected boolean performWorkOnFail;

  protected IOCheck<RET> checkFunction = Optional::empty;

  protected final IOCall<RET> workFunction;

  protected Cooperation2Builder(final IOCall<RET> workFunction) {
    this.workFunction = checkNotNull(workFunction, "The work function for this co-operation is missing");
  }

  @Override
  public Cooperation2Builder<RET> checkFunction(final IOCheck<RET> checkFunction) {
    this.checkFunction = checkNotNull(checkFunction, "The check function for this co-operation is missing");
    return this;
  }

  @Override
  public Cooperation2Builder<RET> performWorkOnFail(final boolean performWorkOnFail) {
    this.performWorkOnFail = performWorkOnFail;
    return this;
  }
}
