
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.Condition;

/**
 * A condition that is always satisfied.
 *
 * @since capabilities 2.0
 */
public class SatisfiedCondition
    implements Condition
{

  private final String reason;

  public SatisfiedCondition(final String reason) {
    this.reason = reason;
  }

  @Override
  public boolean isSatisfied() {
    return true;
  }

  @Override
  public SatisfiedCondition bind() {
    // do nothing
    return this;
  }

  @Override
  public SatisfiedCondition release() {
    // do nothing
    return this;
  }

  @Override
  public String toString() {
    return reason;
  }

  @Override
  public String explainSatisfied() {
    return reason;
  }

  @Override
  public String explainUnsatisfied() {
    return "Not " + reason;
  }

}
