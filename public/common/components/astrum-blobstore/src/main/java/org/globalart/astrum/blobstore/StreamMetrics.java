
package org.globalart.astrum.blobstore;

public class StreamMetrics
{
  private final long size;

  private final String sha1;

  public StreamMetrics(final long size, final String sha1) {
    this.size = size;
    this.sha1 = sha1;
  }

  public long getSize() {
    return size;
  }

  public String getSha1() {
    return sha1;
  }

  @Override
  public String toString() {
    return "StreamMetrics{" +
        "size=" + size +
        ", sha1='" + sha1 + '\'' +
        '}';
  }
}
