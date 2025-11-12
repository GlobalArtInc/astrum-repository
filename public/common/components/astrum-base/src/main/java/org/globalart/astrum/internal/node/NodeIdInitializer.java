
package org.globalart.astrum.internal.node;

/**
 * Initialization or upgrade steps need for NodeId. NodeId is needed very early in the startup sequence so even
 * one time actions must be done here rather than by an upgrade process.
 */
public interface NodeIdInitializer
{
  void initialize();
}
