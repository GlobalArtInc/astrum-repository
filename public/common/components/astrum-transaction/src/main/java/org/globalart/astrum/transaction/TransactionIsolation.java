
package org.globalart.astrum.transaction;

public enum TransactionIsolation
{
  STANDARD,

  /**
   * Serializable isolation level requires that the statements must be executed as if there were no other interleaved
   * transactions. This can lead to concurrency issues under load and stress testing should be performed before using
   * level.
   *
   * See also {@link SerializedAccessException} which will be thrown on failure.
   */
  SERIALIZABLE
}
