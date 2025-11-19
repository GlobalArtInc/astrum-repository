
package org.globalart.astrum.outbound.context;

import java.util.HashMap;
import java.util.Map;

/**
 * ThreadLocal context for sharing outbound request data across modules.
 *
 * This allows passing data between modules that can't directly depend on each other,
 * particularly for tracking outbound request timing
 *
 *
 */
public class OutboundRequestContext
{
  private static final ThreadLocal<Map<String, Object>> CONTEXT =
      ThreadLocal.withInitial(HashMap::new);

  public static final String DOWNLOAD_TIME = "request.download_time";

  public static final String FORMATTED_STRING = "request.formatted_string";

  public static final String TIMESTAMP_PLACEHOLDER = "__TIMESTAMP__";

  public static final String ELAPSED_TIME_PLACEHOLDER = "__ELAPSED_TIME__";

  public static void remove() {
    CONTEXT.remove();
  }

  private static Map<String, Object> context() {
    return CONTEXT.get();
  }

  public static void setDownloadTime(long timeInMs) {
    context().put(DOWNLOAD_TIME, timeInMs);
  }

  public static void setFormattedString(String request) {
    context().put(FORMATTED_STRING, request);
  }

  public static String getFormattedString() {
    return (String) context().get(FORMATTED_STRING);
  }

  public static Long getDownloadTime() {
    return (Long) context().get(DOWNLOAD_TIME);
  }

  public static int getContextMapSize() {
    return context().size();
  }
}
