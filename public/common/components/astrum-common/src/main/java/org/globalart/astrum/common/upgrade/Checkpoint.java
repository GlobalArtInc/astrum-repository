
package org.globalart.astrum.common.upgrade;

/**
 * Checkpoints data/resources for a specific model/schema before upgrade starts.
 * 
 * @since 3.1
 */
public interface Checkpoint
{
  /**
   * Creates a new checkpoint for the installed version.
   */
  void begin(String version) throws Exception;

  /**
   * Commits the current checkpoint.
   */
  void commit() throws Exception;

  /**
   * Reverts to the previous checkpoint.
   */
  void rollback() throws Exception;

  /**
   * Finishes the checkpoint after the entire upgrade was successfully committed, i.e. all checkpoints succeeded and no
   * {@link #rollback()} will happen. This provides an opportunity to delete temporary/backup files. Any exception
   * thrown will be ignored and does not fail the upgrade.
   */
  void end();
}
