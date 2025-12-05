
package org.globalart.astrum.internal.atlas;

import java.io.IOException;
import java.io.OutputStream;
import java.util.zip.Deflater;
import java.util.zip.DeflaterOutputStream;
import java.util.zip.ZipOutputStream;

/**
 * {@link ZipOutputStream} which has {@link DeflaterOutputStream#syncFlush} exposed.
 *
 * @since 2.7
 */
public class FlushableZipOutputStream
    extends ZipOutputStream
{
  private boolean syncFlush;

  public FlushableZipOutputStream(final OutputStream out) {
    super(out);
  }

  public void setSyncFlush(final boolean syncFlush) {
    this.syncFlush = syncFlush;
  }

  /**
   * Copied (unmodified sans formatting) from {@link DeflaterOutputStream#flush()}.
   */
  public void flush() throws IOException {
    if (syncFlush && !def.finished()) {
      int len = 0;
      while ((len = def.deflate(buf, 0, buf.length, Deflater.SYNC_FLUSH)) > 0) {
        out.write(buf, 0, len);
        if (len < buf.length) {
          break;
        }
      }
    }
    out.flush();
  }
}
