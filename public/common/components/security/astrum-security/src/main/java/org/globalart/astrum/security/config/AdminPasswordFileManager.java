
package org.globalart.astrum.security.config;

import java.io.IOException;

/**
 * @since 3.17
 */
public interface AdminPasswordFileManager
{
  boolean exists();

  String getPath();

  boolean writeFile(String password) throws IOException;

  String readFile() throws IOException;

  void removeFile();
}
