
package org.globalart.astrum.repository.proxy;

import java.io.IOException;
import java.io.InputStream;

import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import static com.google.common.io.ByteStreams.toByteArray;

/**
 * Temporary reusable {@link Content}; caches the original remote content in-memory.
 *
 * @since 3.4
 */
class TempContent
    extends Content
{
  public TempContent(final Content remote) throws IOException {
    super(cachePayload(remote), remote.getAttributes());
  }

  private static Payload cachePayload(final Content remote) throws IOException {
    try (InputStream in = remote.openInputStream()) {
      return new BytesPayload(toByteArray(in), remote.getContentType());
    }
  }
}
