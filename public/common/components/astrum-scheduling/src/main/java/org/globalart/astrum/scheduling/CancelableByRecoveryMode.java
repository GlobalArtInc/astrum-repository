
package org.globalart.astrum.scheduling;

/**
 * Marker interface for task types that should be cancelled when recovery mode is enabled.
 */
public interface CancelableByRecoveryMode
    extends Cancelable
{
}
