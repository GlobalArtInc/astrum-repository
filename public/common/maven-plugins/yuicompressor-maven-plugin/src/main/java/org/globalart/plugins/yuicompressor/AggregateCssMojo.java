
package org.globalart.plugins.yuicompressor;

import java.io.File;
import java.io.IOException;
import java.io.Reader;
import java.io.Writer;

import com.yahoo.platform.yui.compressor.CssCompressor;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;

import static org.apache.maven.plugins.annotations.LifecyclePhase.PROCESS_RESOURCES;

/**
 * Aggregate CSS sources.
 *
 * @since 1.0
 */
@Mojo(name="aggregate-css", defaultPhase = PROCESS_RESOURCES)
public class AggregateCssMojo
    extends AggregateMojoSupport
{
  public static final String[] DEFAULT_INCLUDES = { "**/*.css" };

  @Parameter(defaultValue = "${project.basedir}/src/main/css")
  private File sourceDirectory;

  @Parameter(defaultValue = "${project.build.outputDirectory}/${project.artifactId}-all.css")
  private File output;

  @Override
  protected void processSourceFile(File source, Reader in, Writer buf) throws IOException {
    CssCompressor compressor = new CssCompressor(in);
    compressor.compress(buf, linebreakpos);
  }

  @Override
  protected String[] getDefaultIncludes() {
    return DEFAULT_INCLUDES;
  }

  @Override
  protected File getOutput() {
    return output;
  }

  @Override
  protected File getSourceDirectory() {
    return sourceDirectory;
  }
}
