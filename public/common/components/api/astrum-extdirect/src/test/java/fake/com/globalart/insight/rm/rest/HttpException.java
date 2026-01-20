
package fake.com.globalart.insight.rm.rest;

import java.io.IOException;

public class HttpException
    extends IOException
{
  public HttpException(final int status, final String reason, final Throwable cause) {
    super("Error code " + status + ": " + reason, cause);
  }
}
