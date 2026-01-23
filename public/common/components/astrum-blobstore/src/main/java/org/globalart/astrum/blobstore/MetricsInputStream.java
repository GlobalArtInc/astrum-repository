
package org.globalart.astrum.blobstore;

import java.io.FilterInputStream;
import java.io.InputStream;
import java.security.DigestInputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;

import com.google.common.io.BaseEncoding;
import com.google.common.io.CountingInputStream;

/**
 * A utility to collect metrics about the content of an input stream.
 *
 * @since 3.0
 */
public class MetricsInputStream
    extends FilterInputStream
{
  private final MessageDigest messageDigest;

  private final CountingInputStream countingInputStream;

  public MetricsInputStream(final InputStream input) {
    this(new CountingInputStream(input), createSha1());
  }

  private MetricsInputStream(final CountingInputStream countingStream, final MessageDigest messageDigest) {
    super(new DigestInputStream(countingStream, messageDigest));
    this.messageDigest = messageDigest;
    this.countingInputStream = countingStream;
  }

  private static final BaseEncoding HEX = BaseEncoding.base16().lowerCase();

  public String getMessageDigest() {
    return HEX.encode(messageDigest.digest());
  }

  public long getSize() {
    return countingInputStream.getCount();
  }

  public StreamMetrics getMetrics() {
    return new StreamMetrics(getSize(), getMessageDigest());
  }

  private static MessageDigest createSha1() {
    try {
      return MessageDigest.getInstance("SHA1");
    }
    catch (NoSuchAlgorithmException e) {
      // should never happen
      throw new RuntimeException(e);
    }
  }
}
