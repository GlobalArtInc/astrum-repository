
package org.globalart.astrum.repository.apt.internal.snapshot;

import java.io.FilterInputStream;
import java.io.IOException;

import org.bouncycastle.bcpg.ArmoredInputStream;

/**
 * @since 3.31
 */
public class AptFilterInputStream
    extends FilterInputStream
{
  private boolean done = false;

  private final ArmoredInputStream armoredIn;

  public AptFilterInputStream(final ArmoredInputStream armoredIn) {
    super(armoredIn);
    this.armoredIn = armoredIn;
  }

  @Override
  public int read() throws IOException {
    if (done) {
      return -1;
    }
    int c = armoredIn.read();
    if (c < 0 || !armoredIn.isClearText()) {
      done = true;
      return -1;
    }
    return c;
  }

  @Override
  public int read(byte[] b, int off, int len) throws IOException {
    for (int i = 0; i < len; i++) {
      int c = read();
      if (c == -1) {
        return i == 0 ? -1 : i;
      }
      b[off + i] = (byte) c;
    }
    return len;
  }
}
