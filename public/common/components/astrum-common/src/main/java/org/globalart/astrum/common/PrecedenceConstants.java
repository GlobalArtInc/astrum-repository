
package org.globalart.astrum.common;

/**
 * Default precedence value for components without an explicit
 * {@link org.springframework.core.annotation.Order @Order} annotation.
 *
 * Used by {@link QualifierUtil} and
 * {@link org.globalart.astrum.extender.AstrumLifecycleManager AstrumLifecycleManager}
 * when sorting components that have no ordering annotation.
 */
public final class PrecedenceConstants
{
  /**
   * Default precedence for components without an explicit {@link org.springframework.core.annotation.Order} annotation.
   * Matches the historical default where unannotated components sorted at {@code 0}.
   */
  public static final int DEFAULT_PRECEDENCE = 0;

  private PrecedenceConstants() {
    // prevent instantiation
  }
}
