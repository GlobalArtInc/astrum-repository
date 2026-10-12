package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.FilterOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;

final class DigestingOutputStream
    extends FilterOutputStream
{
  private final MessageDigest digest;

  private long count;

  DigestingOutputStream(final OutputStream out) {
    super(out);
    this.digest = sha256();
  }

  @Override
  public void write(final int b) throws IOException {
    out.write(b);
    digest.update((byte) b);
    count++;
  }

  @Override
  public void write(final byte[] bytes, final int offset, final int length) throws IOException {
    out.write(bytes, offset, length);
    digest.update(bytes, offset, length);
    count += length;
  }

  long count() {
    return count;
  }

  String hex() {
    return HexFormat.of().formatHex(digest.digest());
  }

  static MessageDigest sha256() {
    try {
      return MessageDigest.getInstance("SHA-256");
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }
}
