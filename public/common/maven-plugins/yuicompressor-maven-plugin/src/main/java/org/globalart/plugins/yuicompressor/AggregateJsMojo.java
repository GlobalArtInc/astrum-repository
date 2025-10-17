
package org.globalart.plugins.yuicompressor;

import java.io.File;
import java.io.IOException;
import java.io.Reader;
import java.io.Writer;


import com.yahoo.platform.yui.compressor.JavaScriptCompressor;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;
import org.mozilla.javascript.EvaluatorException;

import static org.apache.maven.plugins.annotations.LifecyclePhase.PROCESS_RESOURCES;

/**
 * Aggregate javascript sources.
 *
 * @since 1.0
 */
@Mojo(name="aggregate-js", defaultPhase = PROCESS_RESOURCES)
public class AggregateJsMojo
    extends AggregateMojoSupport
{
  public static final String[] DEFAULT_INCLUDES = { "**/*.js" };

  @Parameter(defaultValue = "${basedir}/src/main/js")
  private File sourceDirectory;

  @Parameter(defaultValue = "${project.build.outputDirectory}/${project.artifactId}-all.js")
  private File output;

  /**
   * Minify only, do not obfuscate.
   */
  @Parameter(property = "maven.yuicompressor.nomunge", defaultValue = "false")
  private boolean nomunge;

  /**
   * Preserve unnecessary semicolons.
   */
  @Parameter(property = "maven.yuicompressor.preserveAllSemiColons", defaultValue = "false")
  private boolean preserveAllSemiColons;

  /**
   * Disable all micro optimizations.
   */
  @Parameter(property = "maven.yuicompressor.disableOptimizations", defaultValue = "false")
  private boolean disableOptimizations;

  /**
   * Display possible errors in the code.
   */
  @Parameter(property = "maven.yuicompressor.jswarm", defaultValue = "true")
  private boolean jswarn;

  /**
   * YUI javascript compressor custom error reporting.
   */
  private class ErrorReporter
      implements org.mozilla.javascript.ErrorReporter
  {
    private final File source;

    public ErrorReporter(File source) {
      this.source = source;
    }

    private String location(int line, int column) {
      return source + "[" + line + ":" + column + "]: ";
    }

    public void error(String message, String sourceName, int line, String lineSource, int lineOffset) {
      getLog().error(location(line, lineOffset) + message);
    }

    public void warning(String message, String sourceName, int line, String lineSource, int lineOffset) {
      getLog().warn(location(line, lineOffset) + message);
    }

    public EvaluatorException runtimeError(String message, String sourceName, int line, String lineSource,
                                           int lineOffset)
    {
      getLog().error(location(line, lineOffset) + message);
      throw new EvaluatorException(message, sourceName, line, lineSource, lineOffset);
    }
  }

  @Override
  protected void processSourceFile(File source, Reader in, Writer buf) throws IOException {
    JavaScriptCompressor compressor = new JavaScriptCompressor(in, new ErrorReporter(source));
    compressor.compress(buf, linebreakpos, !nomunge, jswarn, preserveAllSemiColons, disableOptimizations);
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
