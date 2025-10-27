
package org.globalart.astrum.common.io;

import java.time.Duration;

import org.globalart.astrum.common.lifecycle.LifecycleSupport;

/**
 * Common scaffolding for {@link CooperationFactory} implementations.
 *
 * @since 3.14
 */
public abstract class CooperationFactorySupport
    extends LifecycleSupport
    implements CooperationFactory
{
  @Override
  public Builder configure() {
    return new MutableConfig();
  }

  /**
   * Builds a new {@link Cooperation} point with the given configuration.
   *
   * @param id unique identifier for this cooperation point
   */
  protected abstract Cooperation build(String id, Config config);

  /**
   * Configuration holder for {@link Cooperation} points.
   */
  public static class Config
  {
    protected int majorTimeoutSeconds = 0;

    protected int minorTimeoutSeconds = 0;

    protected int threadsPerKey = 0;

    public Duration majorTimeout() {
      return Duration.ofSeconds(majorTimeoutSeconds);
    }

    public Duration minorTimeout() {
      return Duration.ofSeconds(minorTimeoutSeconds);
    }

    public int threadsPerKey() {
      return threadsPerKey;
    }

    protected Config copy() {
      Config copy = new Config();
      copy.majorTimeoutSeconds = majorTimeoutSeconds;
      copy.minorTimeoutSeconds = minorTimeoutSeconds;
      copy.threadsPerKey = threadsPerKey;
      return copy;
    }
  }

  /**
   * Mutable {@link Builder} of {@link Config}s.
   */
  private final class MutableConfig
      extends Config
      implements Builder
  {
    @Override
    public Builder majorTimeout(final Duration majorTimeout) {
      this.majorTimeoutSeconds = (int) majorTimeout.getSeconds();
      return this;
    }

    @Override
    public Builder minorTimeout(final Duration minorTimeout) {
      this.minorTimeoutSeconds = (int) minorTimeout.getSeconds();
      return this;
    }

    @Override
    public Builder threadsPerKey(final int threadsPerKey) {
      this.threadsPerKey = threadsPerKey;
      return this;
    }

    @Override
    public Cooperation build(final String id) {
      return CooperationFactorySupport.this.build(id, copy());
    }
  }
}
