
package org.globalart.astrum.repository.http;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;

import com.google.common.collect.Range;

import static com.google.common.io.ByteStreams.limit;

/**
 * A wrapper {@link Payload} that returns only a portion of the original payload.
 *
 * @since 3.0
 */
class PartialPayload
    implements Payload
{
  private final Payload payload;

  private final Range<Long> rangeToSend;

  private final long partialSize;

  /**
   * The endpoints of the Range are interpreted as the first and last byte positions to send.
   */
  public PartialPayload(final Payload payload, final Range<Long> rangeToSend) {
    this.payload = payload;
    this.rangeToSend = rangeToSend;
    this.partialSize = 1 + rangeToSend.upperEndpoint() - rangeToSend.lowerEndpoint();
  }

  @Override
  public InputStream openInputStream() throws IOException {
    final InputStream payloadStream = payload.openInputStream();
    payloadStream.skip(rangeToSend.lowerEndpoint());
    return limit(payloadStream, partialSize);
  }

  @Override
  public long getSize() {
    return partialSize;
  }

  @Nullable
  @Override
  public String getContentType() {
    return payload.getContentType();
  }
}
