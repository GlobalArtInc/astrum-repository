
package org.globalart.astrum.thread.internal;

import java.util.Map;

import org.globalart.astrum.security.UserIdMdcHelper;

import com.google.common.collect.Maps;
import org.slf4j.MDC;

/**
 * Simple helper class to manipulate MDC.
 *
 * @since 2.6
 */
public class MDCUtils
{
  private MDCUtils() {
    // empty
  }

  public static final String CONTEXT_NON_INHERITABLE_KEY = "non-inheritable";

  public static Map<String, String> getCopyOfContextMap() {
    final boolean inheritable = MDC.get(CONTEXT_NON_INHERITABLE_KEY) == null;
    Map<String, String> result = null;
    if (inheritable) {
      // noinspection unchecked
      result = MDC.getCopyOfContextMap();
    }
    if (result == null) {
      result = Maps.newHashMap();
    }
    result.remove(CONTEXT_NON_INHERITABLE_KEY);
    return result;
  }

  public static void setContextMap(Map<String, String> context) {
    if (context != null) {
      MDC.setContextMap(context);
      UserIdMdcHelper.setIfNeeded();
    }
    else {
      MDC.clear();
      UserIdMdcHelper.set();
    }
  }
}
