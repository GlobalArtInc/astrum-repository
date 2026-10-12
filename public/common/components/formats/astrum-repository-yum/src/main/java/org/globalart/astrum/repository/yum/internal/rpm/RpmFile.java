package org.globalart.astrum.repository.yum.internal.rpm;

public record RpmFile(String path, String type)
{
  public static final String FILE = "";

  public static final String DIR = "dir";

  public static final String GHOST = "ghost";

  public boolean isPrimary() {
    return path.startsWith("/etc/") || path.contains("bin/") || "/usr/lib/sendmail".equals(path);
  }
}
