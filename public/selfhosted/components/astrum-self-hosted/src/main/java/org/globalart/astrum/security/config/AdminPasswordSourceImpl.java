
package org.globalart.astrum.security.config;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.UUID;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.text.Strings2;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class AdminPasswordSourceImpl
    implements AdminPasswordSource
{
  public static final String DEFAULT_PASSWORD = "admin123";

  private final AdminPasswordFileManager adminPasswordFileManager;

  @Autowired
  public AdminPasswordSourceImpl(
      final AdminPasswordFileManager adminPasswordFileManager)
  {
    this.adminPasswordFileManager = checkNotNull(adminPasswordFileManager);
  }

  public String getPassword(final boolean randomPassword) {
    try {
      String savedPassword = adminPasswordFileManager.readFile();

      if (!Strings2.isBlank(savedPassword)) {
        return savedPassword;
      }
      else if (!randomPassword) {
        return DEFAULT_PASSWORD;
      }

      savedPassword = UUID.randomUUID().toString();

      // failure writing file to disk, revert to using default
      if (!adminPasswordFileManager.writeFile(savedPassword)) {
        savedPassword = DEFAULT_PASSWORD;
      }
      return savedPassword;
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }
}
