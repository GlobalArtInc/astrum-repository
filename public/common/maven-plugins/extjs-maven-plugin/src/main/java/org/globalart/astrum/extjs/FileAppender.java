
package org.globalart.astrum.extjs;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;
import java.io.IOException;
import java.io.Writer;

import org.apache.maven.plugin.logging.Log;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * File appender.
 *
 * @since 3.0
 */
public class FileAppender
{
  protected final Log log;

  protected final Writer output;

  public FileAppender(final Log log, final Writer output) {
    this.log = checkNotNull(log);
    this.output = checkNotNull(output);
  }

  protected File source;

  protected long linenum;

  public void append(final File source) throws IOException {
    this.source = checkNotNull(source);
    this.linenum = 0;

    log.debug("Appending: " + source);

    BufferedReader input = new BufferedReader(new FileReader(source));
    try {
      String line;
      while ((line = input.readLine()) != null) {
        linenum++;
        append(line);
      }
    }
    finally {
      input.close();
    }

    output.write('\n');
  }

  protected void append(final String line) throws IOException {
    output.write(line);
    output.write('\n');
  }
}
