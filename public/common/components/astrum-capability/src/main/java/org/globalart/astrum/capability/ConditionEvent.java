
package org.globalart.astrum.capability;

/**
 * {@link Condition} related events.
 *
 * @since capabilities 2.0
 */
public class ConditionEvent
{

  private final Condition condition;

  public ConditionEvent(final Condition condition) {
    this.condition = condition;
  }

  public Condition getCondition() {
    return condition;
  }

  @Override
  public String toString() {
    return getCondition().toString();
  }

  /**
   * Event fired when a condition becomes satisfied.
   *
   * @since capabilities 2.0
   */
  public static class Satisfied
      extends ConditionEvent
  {

    public Satisfied(final Condition condition) {
      super(condition);
    }

    @Override
    public String toString() {
      return super.toString() + " is satisfied";
    }

  }

  /**
   * Event fired when a condition becomes unsatisfied.
   *
   * @since capabilities 2.0
   */
  public static class Unsatisfied
      extends ConditionEvent
  {

    public Unsatisfied(final Condition condition) {
      super(condition);
    }

    @Override
    public String toString() {
      return super.toString() + " is unsatisfied";
    }

  }

}
