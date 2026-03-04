
package org.globalart.astrum.selector;

/**
 * Represents runtime exceptions from evaluation an otherwise valid selector expression
 * 
 * @since 3.1
 */
public class SelectorEvaluationException
    extends Exception
{
  public SelectorEvaluationException(String message) {
    super(message);
  }

  public SelectorEvaluationException(String message, Exception cause) {
    super(message, cause);
  }
}
