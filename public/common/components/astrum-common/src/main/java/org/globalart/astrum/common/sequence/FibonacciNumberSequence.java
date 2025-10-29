
package org.globalart.astrum.common.sequence;

/**
 * Class that calculates "fibonacci-like" sequence.
 *
 * For those uninformed, there is only one Fibonacci sequence out there, all others are just alike.
 * Fibonacci sequence per-definition starts with 0, 1, 1... It does not start with 10.
 * Or, the sequence that starts with 10 is NOT a Fibonacci sequence.
 *
 * This implementation avoids the "artificial" leading 0 in the sequence.
 */
public class FibonacciNumberSequence
    implements NumberSequence
{
  private final long startA;

  private final long startB;

  private long a;

  private long b;

  public FibonacciNumberSequence() {
    this(1);
  }

  public FibonacciNumberSequence(long start) {
    this(start, start);
  }

  public FibonacciNumberSequence(long startA, long startB) {
    this.startA = startA;
    this.startB = startB;
    reset();
  }

  public long next() {
    long res = a;

    a = b;

    b = res + b;

    return res;
  }

  public long prev() {
    long tmp = b;

    b = a;

    a = tmp - b;

    return a;
  }

  public long peek() {
    return a;
  }

  public void reset() {
    a = startA;
    b = startB;
  }

}
