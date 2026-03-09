
package org.globalart.astrum.script;

import javax.annotation.Nullable;

import org.globalart.astrum.common.lifecycle.Lifecycle;

/**
 * Script manager.
 *
 * @since 3.0
 */
public interface ScriptManager
    extends Lifecycle
{
  public static final String DEFAULT_TYPE = "groovy";

  Iterable<Script> browse();

  @Nullable
  Script get(String name);

  Script create(String name, String content, String type);

  Script update(String name, String content);

  void delete(String name);

  boolean isEnabled();
}
