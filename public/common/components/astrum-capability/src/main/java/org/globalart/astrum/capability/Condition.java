
package org.globalart.astrum.capability;

/**
 * An evaluable condition.
 */
public interface Condition
    extends Evaluable
{

  /**
   * Binds (eventual) resources used by condition. Before binding, condition should not be used.
   * <p/>
   * Calling this method multiple times should not fail, eventually should log a warning.
   *
   * @return itself, for fluent api usage
   */
  Condition bind();

  /**
   * Releases (eventual) resources used by condition. After releasing, condition should not be used until not binding
   * it again.
   * <p/>
   * Calling this method multiple times should not fail, eventually should log a warning.
   *
   * @return itself, for fluent api usage
   */
  Condition release();

}
