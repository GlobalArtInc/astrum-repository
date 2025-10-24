
package org.globalart.astrum.common.cooperation2.internal;

import java.io.IOException;
import java.util.Collections;
import java.util.Map;

import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.CooperationKey;
import org.globalart.astrum.common.cooperation2.IOCall;

import com.google.common.base.Stopwatch;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * An implementation of {@link Cooperation2} which doesn't use any concurrency control and each thread proceeds
 * individually. This is used when co-operation is disabled.
 *
 * @since 3.41
 */
public class DisabledCooperation2
    implements Cooperation2
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String scope;

  public DisabledCooperation2(final String scope) {
    this.scope = scope;
  }

  @Override
  public <RET> Builder<RET> on(final IOCall<RET> workFunction) {
    return new DisabledCooperation2Builder<>(workFunction);
  }

  @Override
  public Map<String, Integer> getThreadCountPerKey() {
    return Collections.emptyMap();
  }

  private class DisabledCooperation2Builder<R>
      extends Cooperation2Builder<R>
  {
    DisabledCooperation2Builder(final IOCall<R> workFunction) {
      super(workFunction);
    }

    @Override
    public R cooperate(final String action, final String... nestedScope) throws IOException {
      CooperationKey cooperationKey = CooperationKey.create(scope, action, nestedScope);
      log.debug("Starting work for {}", cooperationKey);
      Stopwatch timer = Stopwatch.createStarted();

      try {
        return workFunction.call();
      }
      finally {
        log.debug("Completed work for {} in {}", cooperationKey, timer.elapsed());
      }
    }
  }
}
