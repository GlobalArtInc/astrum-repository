package org.globalart.astrum.repository.alpine.internal.apk;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;

public final class ApkPackageParser
{
  private static final long MAX_CONTROL_SIZE = 64L * 1024 * 1024;

  private static final String PKGINFO = ".PKGINFO";

  private static final String SIGNATURE_PREFIX = ".SIGN.";

  private ApkPackageParser() {
  }

  public static ApkPackage parse(final InputStream in, final long fileSize) throws IOException {
    GzipMembers members = new GzipMembers(in, MAX_CONTROL_SIZE);
    GzipMembers.Member first = members.next();
    if (first == null) {
      throw new IOException("Empty package");
    }
    GzipMembers.Member control = first;
    Map<String, byte[]> entries = tarEntries(first.inflated());
    if (entries.keySet().stream().allMatch(name -> name.startsWith(SIGNATURE_PREFIX)) && !entries.isEmpty()) {
      control = members.next();
      if (control == null) {
        throw new IOException("Package has no control segment");
      }
      entries = tarEntries(control.inflated());
    }
    byte[] pkginfo = entries.get(PKGINFO);
    if (pkginfo == null) {
      throw new IOException("Package has no .PKGINFO");
    }
    Map<String, List<String>> info = parsePkgInfo(new String(pkginfo, StandardCharsets.UTF_8));
    String checksum = "Q1" + Base64.getEncoder().encodeToString(sha1(control.raw()));
    return ApkPackage.fromPkgInfo(info, checksum, fileSize);
  }

  static Map<String, List<String>> parsePkgInfo(final String text) {
    Map<String, List<String>> info = new LinkedHashMap<>();
    for (String line : text.split("\n")) {
      String trimmed = line.strip();
      if (trimmed.isEmpty() || trimmed.startsWith("#")) {
        continue;
      }
      int separator = trimmed.indexOf('=');
      if (separator <= 0) {
        continue;
      }
      String key = trimmed.substring(0, separator).strip();
      String value = trimmed.substring(separator + 1).strip();
      info.computeIfAbsent(key, k -> new ArrayList<>()).add(value);
    }
    return info;
  }

  static Map<String, byte[]> tarEntries(final byte[] tar) throws IOException {
    Map<String, byte[]> entries = new LinkedHashMap<>();
    try (TarArchiveInputStream in = new TarArchiveInputStream(new ByteArrayInputStream(tar))) {
      TarArchiveEntry entry;
      while ((entry = in.getNextEntry()) != null) {
        if (entry.isFile()) {
          entries.put(entry.getName(), in.readAllBytes());
        }
      }
    }
    catch (IOException e) {
      if (entries.isEmpty()) {
        throw e;
      }
    }
    return entries;
  }

  static byte[] sha1(final byte[] data) {
    try {
      return MessageDigest.getInstance("SHA-1").digest(data);
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }
}
