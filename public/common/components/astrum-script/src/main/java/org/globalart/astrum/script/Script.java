
package org.globalart.astrum.script;

/**
 * A user created script
 *
 * @since 3.20
 */
public interface Script
{
  String getName();

  void setName(final String name);

  String getContent();

  void setContent(final String content);

  String getType();

  void setType(final String type);
}
