
package org.globalart.astrum.security.config;

public interface AdminPasswordSource
{
  String getPassword(final boolean randomPassword);
}
