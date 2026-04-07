package org.globalart.astrum.repository.docker.internal.v2;

import java.util.regex.Pattern;

import javax.annotation.Nullable;

import com.google.common.hash.HashCode;
import com.google.common.hash.Hashing;

public final class DockerDigest
{
  public static final String SHA256 = "sha256";

  private static final Pattern DIGEST = Pattern.compile("^[a-z0-9]+(?:[.+_-][a-z0-9]+)*:[a-zA-Z0-9=_-]+$");

  private static final Pattern SHA256_DIGEST = Pattern.compile("^sha256:[a-f0-9]{64}$");

  private DockerDigest() {
  }

  public static boolean isDigest(@Nullable final String value) {
    return value != null && DIGEST.matcher(value).matches();
  }

  public static boolean isSupported(@Nullable final String value) {
    return value != null && SHA256_DIGEST.matcher(value).matches();
  }

  public static String sha256(final byte[] bytes) {
    return SHA256 + ":" + Hashing.sha256().hashBytes(bytes).toString();
  }

  public static String sha256(final HashCode hashCode) {
    return SHA256 + ":" + hashCode.toString();
  }

  public static String hex(final String digest) {
    int index = digest.indexOf(':');
    return index < 0 ? digest : digest.substring(index + 1);
  }
}
