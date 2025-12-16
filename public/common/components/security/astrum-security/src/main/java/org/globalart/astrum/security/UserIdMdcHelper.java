
package org.globalart.astrum.security;

import com.google.common.base.Strings;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MDC;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.security.UserIdHelper.SYSTEM;
import static org.globalart.astrum.security.UserIdHelper.UNKNOWN;

/**
 * Helper to set the {@code userId} MDC attribute.
 *
 * @since 2.7.2
 */
public class UserIdMdcHelper
{
  private UserIdMdcHelper() {
    // empty
  }

  private static final Logger log = LoggerFactory.getLogger(UserIdMdcHelper.class);

  public static final String KEY = "userId";

  public static boolean isSet() {
    String userId = MDC.get(KEY);
    return !(Strings.isNullOrEmpty(userId) || UNKNOWN.equals(userId));
  }

  public static void setIfNeeded() {
    if (!isSet()) {
      set();
    }
  }

  public static void set(final Subject subject) {
    checkNotNull(subject);
    String userId = UserIdHelper.get(subject);
    log.trace("Set: {}", userId);
    MDC.put(KEY, userId);
  }

  public static void set() {
    MDC.put(KEY, UserIdHelper.get());
  }

  /**
   * @since 3.0
   */
  public static void unknown() {
    MDC.put(KEY, UNKNOWN);
  }

  /**
   * @since 3.0
   */
  public static void system() {
    MDC.put(KEY, SYSTEM);
  }

  public static void unset() {
    MDC.remove(KEY);
  }
}
