
package org.globalart.astrum.repository.view.payloads;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Stream payload.
 *
 * @since 3.0
 */
public class StreamPayload
    implements Payload
{
  private final InputStreamSupplier stream;

  private final long size;

  private final String contentType;

  public StreamPayload(final InputStreamSupplier stream, final long size, @Nullable final String contentType) {
    this.stream = checkNotNull(stream);
    this.size = size;
    this.contentType = contentType;
  }

  /**
   * Returnes opened stream from configured supplier.
   */
  @Override
  public InputStream openInputStream() throws IOException {
    return stream.get();
  }

  @Override
  public long getSize() {
    return size;
  }

  @Nullable
  @Override
  public String getContentType() {
    return contentType;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "stream=" + stream +
        ", size=" + size +
        ", contentType='" + contentType + '\'' +
        '}';
  }

  //
  // Supplier
  //

  /**
   * Returns an {@link InputStream} and takes in an argument
   */
  public interface InputStreamFunction<T>
  {
    /**
     * Applies this function to the given argument.
     *
     * @param t the function argument
     * @return the function result
     */
    @Nonnull
    InputStream apply(T t) throws IOException;
  }
}
