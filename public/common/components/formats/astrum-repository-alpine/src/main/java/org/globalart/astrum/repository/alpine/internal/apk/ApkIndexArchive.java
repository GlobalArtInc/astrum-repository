package org.globalart.astrum.repository.alpine.internal.apk;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import javax.annotation.Nullable;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;

public final class ApkIndexArchive
{
  public static final String APKINDEX = "APKINDEX";

  public static final String DESCRIPTION = "DESCRIPTION";

  private static final long MAX_INDEX_SIZE = 256L * 1024 * 1024;

  private static final int TAR_EOF_SIZE = 1024;

  private ApkIndexArchive() {
  }

  public static byte[] build(
      final String description,
      final List<ApkIndexEntry> entries,
      @Nullable final ApkSigningKey signingKey,
      final Instant modified) throws IOException
  {
    Map<String, byte[]> files = new LinkedHashMap<>();
    files.put(DESCRIPTION, description.getBytes(StandardCharsets.UTF_8));
    files.put(APKINDEX, ApkIndexEntry.render(entries).getBytes(StandardCharsets.UTF_8));
    byte[] index = GzipMembers.gzip(tar(files, modified, true));
    if (signingKey == null) {
      return index;
    }
    byte[] signature = signingKey.sign(index);
    byte[] signatureSegment =
        GzipMembers.gzip(tar(Map.of(signingKey.signatureEntryName(), signature), modified, false));
    ByteArrayOutputStream out = new ByteArrayOutputStream(signatureSegment.length + index.length);
    out.writeBytes(signatureSegment);
    out.writeBytes(index);
    return out.toByteArray();
  }

  public static Contents read(final InputStream in) throws IOException {
    GzipMembers members = new GzipMembers(in, MAX_INDEX_SIZE);
    GzipMembers.Member first = members.next();
    if (first == null) {
      throw new IOException("Empty index archive");
    }
    Map<String, byte[]> entries = ApkPackageParser.tarEntries(first.inflated());
    String signature = entries.keySet().stream().filter(name -> name.startsWith(".SIGN.")).findFirst().orElse(null);
    if (signature != null && !entries.containsKey(APKINDEX)) {
      GzipMembers.Member second = members.next();
      if (second == null) {
        throw new IOException("Index archive has no index segment");
      }
      entries = ApkPackageParser.tarEntries(second.inflated());
    }
    byte[] apkindex = entries.get(APKINDEX);
    if (apkindex == null) {
      throw new IOException("Index archive has no APKINDEX");
    }
    byte[] description = entries.get(DESCRIPTION);
    return new Contents(
        description == null ? "" : new String(description, StandardCharsets.UTF_8).strip(),
        new String(apkindex, StandardCharsets.UTF_8),
        signature);
  }

  private static byte[] tar(final Map<String, byte[]> files, final Instant modified, final boolean terminate)
      throws IOException
  {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (TarArchiveOutputStream tar = new TarArchiveOutputStream(out, 512)) {
      tar.setLongFileMode(TarArchiveOutputStream.LONGFILE_GNU);
      for (Entry<String, byte[]> file : files.entrySet()) {
        TarArchiveEntry entry = new TarArchiveEntry(file.getKey());
        entry.setSize(file.getValue().length);
        entry.setMode(0100644);
        entry.setUserName("root");
        entry.setGroupName("root");
        entry.setModTime(modified.toEpochMilli());
        tar.putArchiveEntry(entry);
        tar.write(file.getValue());
        tar.closeArchiveEntry();
      }
      tar.finish();
    }
    byte[] bytes = out.toByteArray();
    return terminate ? bytes : Arrays.copyOf(bytes, bytes.length - TAR_EOF_SIZE);
  }

  public record Contents(String description, String apkindex, @Nullable String signature)
  {
    public List<ApkIndexEntry> entries() {
      return ApkIndexEntry.parse(apkindex);
    }
  }
}
