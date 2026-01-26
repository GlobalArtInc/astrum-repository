
package org.globalart.astrum.blobstore;

import java.io.ByteArrayInputStream;
import java.io.InputStream;

import org.junit.Test;

import static com.google.common.io.ByteStreams.copy;
import static com.google.common.io.ByteStreams.nullOutputStream;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;

/**
 * Tests for {@link MetricsInputStream}.
 */
public class MetricsInputStreamTest
{
  @Test
  public void testLength() throws Exception {
    assertThat(measure("ABC".getBytes("UTF-8")).getSize(), is(equalTo(3L)));
    assertThat(measure(new byte[10000]).getSize(), is(equalTo(10000L)));
  }

  @Test
  public void testHashesDiffer() throws Exception {
    final String hash1 = measure("ABC".getBytes("UTF-8")).getMessageDigest();
    final String hash2 = measure(new byte[10000]).getMessageDigest();

    assertThat(hash1, not(equalTo(hash2)));
  }

  @Test
  public void referenceHashMatches() throws Exception {
    final MetricsInputStream measure = measure(
        getClass().getResourceAsStream("sha1_is_2589766c6dac3402cab552602d457e7e8af12efd.bytes"));
    assertThat(measure.getMessageDigest(), is(equalTo("2589766c6dac3402cab552602d457e7e8af12efd")));
  }

  private MetricsInputStream measure(final byte[] testData) throws Exception {
    return measure(new ByteArrayInputStream(testData));
  }

  private MetricsInputStream measure(final InputStream inputStream) throws Exception {
    final MetricsInputStream metricStream = new MetricsInputStream(inputStream);
    copy(metricStream, nullOutputStream());
    return metricStream;
  }
}
