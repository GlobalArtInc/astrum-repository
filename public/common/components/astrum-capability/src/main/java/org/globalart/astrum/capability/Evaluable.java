
package org.globalart.astrum.capability;

/**
 * A logical condition.
 *
 * @since capabilities 2.2
 */
public interface Evaluable
{

  /**
   * Whether or not the condition is satisfied.
   *
   * @return true, if condition is satisfied
   */
  boolean isSatisfied();

  /**
   * Describe condition in case that it is satisfied.
   *
   * @return description
   */
  String explainSatisfied();

  /**
   * Describe condition in case that it is not satisfied.
   *
   * @return description
   */
  String explainUnsatisfied();

}
