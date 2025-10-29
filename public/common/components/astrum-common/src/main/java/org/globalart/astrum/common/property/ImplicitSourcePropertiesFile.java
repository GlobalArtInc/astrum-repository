
package org.globalart.astrum.common.property;

import java.io.IOException;
import java.io.Reader;
import java.util.Properties;

/**
 * Represents a properties file that maintains the state of the {@link Reader}/{@link java.io.Writer}
 * or {@link java.io.InputStream}/{@link java.io.OutputStream} internally and does not require a parameter
 * to {@code load()} or {@code store()}
 *
 * @since 3.15
 */
public abstract class ImplicitSourcePropertiesFile
    extends Properties
{
  public abstract void load() throws IOException;

  public abstract void store() throws IOException;

  public abstract boolean exists() throws IOException;
}
