package org.globalart.astrum.repository.golang.internal.util;

import java.io.IOException;
import java.io.InputStream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;

public record GoModuleZip(String module, String version, @Nullable byte[] goMod)
{
  private static final String GO_MOD = "go.mod";

  private static final int MAX_GO_MOD_SIZE = 16 << 20;

  public static GoModuleZip read(final InputStream in) {
    String prefix = null;
    byte[] goMod = null;
    try (ZipInputStream zip = new ZipInputStream(in)) {
      ZipEntry entry;
      while ((entry = zip.getNextEntry()) != null) {
        String name = entry.getName();
        String entryPrefix = prefixOf(name);
        if (prefix == null) {
          prefix = entryPrefix;
        }
        else if (!prefix.equals(entryPrefix)) {
          throw new InvalidContentException(
              "Module zip mixes " + prefix + " and " + entryPrefix + " entries");
        }
        if (name.equals(prefix + GO_MOD)) {
          goMod = zip.readNBytes(MAX_GO_MOD_SIZE + 1);
          if (goMod.length > MAX_GO_MOD_SIZE) {
            throw new InvalidContentException("go.mod in module zip is too large");
          }
        }
      }
    }
    catch (IOException e) {
      throw new InvalidContentException("Unable to read module zip", e);
    }
    if (prefix == null) {
      throw new InvalidContentException("Module zip is empty or not a zip file");
    }
    int at = prefix.lastIndexOf('@');
    return new GoModuleZip(prefix.substring(0, at), prefix.substring(at + 1, prefix.length() - 1), goMod);
  }

  private static String prefixOf(final String name) {
    int at = name.indexOf('@');
    int slash = at < 0 ? -1 : name.indexOf('/', at);
    if (at <= 0 || slash < 0 || slash == at + 1) {
      throw new InvalidContentException("Module zip entry " + name + " is not inside a module@version directory");
    }
    return name.substring(0, slash + 1);
  }
}
