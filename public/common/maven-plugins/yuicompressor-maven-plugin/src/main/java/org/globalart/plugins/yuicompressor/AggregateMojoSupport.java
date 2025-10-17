
package org.globalart.plugins.yuicompressor;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.Reader;
import java.io.StringWriter;
import java.io.Writer;
import java.util.List;

import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugins.annotations.Parameter;
import org.codehaus.plexus.util.IOUtil;

/**
 * Support for aggregation mojos.
 *
 * @since 1.0
 */
abstract class AggregateMojoSupport
    extends ProcessSourcesMojoSupport
{
  /**
   * Insert line breaks in output after the specified column number.
   */
  @Parameter(property = "maven.yuicompressor.linebreakpos", defaultValue = "0")
  protected int linebreakpos;

  /**
   * Aggregate only, no minification.
   */
  @Parameter(property = "maven.yuicompressor.nominify", defaultValue = "false")
  protected boolean nominify;

  /**
   * Insert new line after each concatenation.
   */
  @Parameter(defaultValue = "true")
  protected boolean insertNewLine;

  @Override
  protected void processSources(List<File> sources) throws MojoExecutionException {
    File output = getOutput();

    // see if there are any changes we need to include in the aggregate
    boolean uptodate = true;
    for (File source : sources) {
      uptodate = output.exists() && source.exists() && output.lastModified() > source.lastModified();
      if (!uptodate) {
        break;
      }
    }

    if (!uptodate) {
      try {
        StringWriter buf = new StringWriter();

        for (File source : sources) {
          Reader in = new BufferedReader(new InputStreamReader(new FileInputStream(source), "UTF-8"));
          try {
            // don't minify, simply write directly out to buffer
            if (nominify) {
              IOUtil.copy(in, buf);
            }
            // compress away then write out to buffer
            else {
              processSourceFile(source, in, buf);
            }
          }
          finally {
            IOUtil.close(in);
          }

          if (insertNewLine) {
            buf.write('\n');
          }
        }

        output.getParentFile().mkdirs();

        getLog().info("Writing: " + output);
        OutputStream out = new FileOutputStream(output);
        try {
          IOUtil.copy(buf.getBuffer().toString(), out);
        }
        finally {
          IOUtil.close(out);
        }
      }
      catch (IOException e) {
        throw new MojoExecutionException("Could not create aggregate file", e);
      }
    }
  }

  protected abstract File getOutput();

  protected abstract void processSourceFile(File source, Reader in, Writer buf) throws IOException;
}
