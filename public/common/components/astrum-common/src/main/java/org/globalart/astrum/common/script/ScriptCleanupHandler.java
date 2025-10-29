
package org.globalart.astrum.common.script;

/**
 * Handler that executes cleanup steps after execution of scripts
 *
 * @since 3.17
 */
public interface ScriptCleanupHandler
{
  void cleanup(String context);
}
