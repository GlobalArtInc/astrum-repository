package org.globalart.astrum.repository.terraform.internal;

import java.net.URI;
import java.security.InvalidKeyException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.time.Clock;
import java.time.Duration;
import java.util.Base64;

import javax.annotation.Nullable;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Time-limited download URLs.
 *
 * Terraform sends its credentials to the registry API only: it fetches the checksums and the signature of a provider
 * without any, and packages with nothing but a {@code .netrc} entry. The locations a repository hands out therefore
 * carry a signature over the path they name and an expiry, made with a secret of the repository; a request that
 * presents a valid one may read that one path until the expiry. Only a user who could read the registry document
 * receives such a location.
 */
public final class DownloadTokens
{
  public static final String EXPIRES = "expires";

  public static final String SIGNATURE = "signature";

  public static final Duration LIFETIME = Duration.ofMinutes(15);

  static final String SECRET_ATTRIBUTE = "terraformDownloadSecret";

  private static final String HMAC = "HmacSHA256";

  private static final SecureRandom RANDOM = new SecureRandom();

  private final byte[] secret;

  private final Clock clock;

  public DownloadTokens(final byte[] secret, final Clock clock) {
    this.secret = secret.clone();
    this.clock = clock;
  }

  /**
   * The tokens of a repository, creating its secret on first use.
   */
  public static DownloadTokens of(final TerraformContentFacet content) {
    String encoded = secret(content);
    if (encoded == null) {
      synchronized (content) {
        encoded = secret(content);
        if (encoded == null) {
          byte[] fresh = new byte[32];
          RANDOM.nextBytes(fresh);
          encoded = Base64.getEncoder().encodeToString(fresh);
          content.attributes(AttributeOperation.SET, SECRET_ATTRIBUTE, encoded);
        }
      }
    }
    return new DownloadTokens(Base64.getDecoder().decode(encoded), Clock.systemUTC());
  }

  @Nullable
  private static String secret(final TerraformContentFacet content) {
    Object stored = content.attributes().get(SECRET_ATTRIBUTE);
    return stored instanceof String value && !value.isBlank() ? value : null;
  }

  /**
   * A URL relative to a document that names {@code targetPath}, with a token for that path appended.
   */
  public String sign(final String relativeUrl, final String targetPath) {
    long expires = clock.instant().plus(LIFETIME).getEpochSecond();
    String separator = relativeUrl.contains("?") ? "&" : "?";
    return relativeUrl + separator + EXPIRES + "=" + expires + "&" + SIGNATURE + "=" + signature(targetPath, expires);
  }

  /**
   * Re-signs a relative URL of a document at {@code documentPath} that another repository signed, so it is valid
   * for this repository. URLs that carry no token, or are absolute, are returned unchanged.
   */
  public String resign(final String url, final String documentPath) {
    int query = url.indexOf('?');
    if (query < 0 || url.contains("://") || !url.substring(query).contains(SIGNATURE + "=")) {
      return url;
    }
    String base = url.substring(0, query);
    String target = targetPath(documentPath, base);
    return target == null ? url : sign(base, target);
  }

  public boolean isValid(final String path, @Nullable final String expires, @Nullable final String signature) {
    if (expires == null || signature == null) {
      return false;
    }
    long expiry;
    try {
      expiry = Long.parseLong(expires);
    }
    catch (NumberFormatException e) {
      return false;
    }
    if (expiry < clock.instant().getEpochSecond()) {
      return false;
    }
    return MessageDigest.isEqual(signature(path, expiry).getBytes(UTF_8), signature.getBytes(UTF_8));
  }

  /**
   * The path inside the repository a relative URL names, without the subdirectory a module location may append.
   */
  @Nullable
  static String targetPath(final String documentPath, final String relativeUrl) {
    String url = relativeUrl;
    int subdir = url.indexOf("//", 1);
    if (subdir > 0) {
      url = url.substring(0, subdir);
    }
    try {
      String path = URI.create("http://repository" + documentPath).resolve(url).getPath();
      return path == null || path.contains("..") ? null : path;
    }
    catch (IllegalArgumentException e) {
      return null;
    }
  }

  private String signature(final String path, final long expires) {
    try {
      Mac mac = Mac.getInstance(HMAC);
      mac.init(new SecretKeySpec(secret, HMAC));
      byte[] digest = mac.doFinal((path + "\n" + expires).getBytes(UTF_8));
      return Base64.getUrlEncoder().withoutPadding().encodeToString(digest);
    }
    catch (NoSuchAlgorithmException | InvalidKeyException e) {
      throw new IllegalStateException(e);
    }
  }
}
