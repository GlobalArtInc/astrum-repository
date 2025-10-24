
package org.globalart.astrum.common.cooperation2.internal;

import java.time.Duration;

import org.globalart.astrum.common.cooperation2.Config;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory.Builder;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.41
 */
public abstract class MutableConfigSupport
    extends Config
    implements Builder
{
  protected boolean enabled = true;

  @Override
  public Builder majorTimeout(final Duration majorTimeout) {
    this.majorTimeoutSeconds = (int) checkNotNull(majorTimeout).getSeconds();
    return this;
  }

  @Override
  public Builder minorTimeout(final Duration minorTimeout) {
    this.minorTimeoutSeconds = (int) checkNotNull(minorTimeout).getSeconds();
    return this;
  }

  @Override
  public Builder threadsPerKey(final int threadsPerKey) {
    this.threadsPerKey = threadsPerKey;
    return this;
  }

  @Override
  public Builder enabled(final boolean enabled) {
    this.enabled = enabled;
    return this;
  }
}
