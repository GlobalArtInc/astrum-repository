

package org.globalart.astrum.audit;

import java.util.HashMap;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.text.Strings2;

import com.google.common.base.Joiner;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkState;

/**
 * Support for auditor implementations.
 *
 * @since 3.1
 */
public abstract class AuditorSupport
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  /**
   * Context value to use for global/system audit data when there is nothing more specific to use.
   */
  protected static final String SYSTEM_CONTEXT = "system";

  /**
   * Common type for created events.
   */
  public static final String CREATED_TYPE = "created";

  /**
   * Common type for updated events.
   */
  public static final String UPDATED_TYPE = "updated";

  /**
   * Common type for deleted events.
   */
  public static final String DELETED_TYPE = "deleted";

  /**
   * Common type for deleted events.
   */
  protected static final String PURGE_TYPE = "purged";

  /**
   * Common type for changed events.
   */
  protected static final String CHANGED_TYPE = "changed";

  /**
   * Helper to join a list into a string for attributes.
   */
  private static final Joiner LIST_JOINER = Joiner.on(", ").skipNulls();

  private AuditRecorder auditRecorder;

  /**
   * Mapping of class to simple type names for auditing.
   */
  private Map<Class, String> typeLookup = new HashMap<>();

  @Autowired
  public void setAuditRecorder(final AuditRecorder auditRecorder) {
    this.auditRecorder = auditRecorder;
  }

  /**
   * Register a simple type name for given class.
   */
  protected void registerType(final Class type, final String name) {
    typeLookup.put(type, name);
  }

  /**
   * Lookup simple type name for given class.
   *
   * If there is no mapping then the class.simpleName will be used.
   */
  protected String type(final Class type) {
    String name = typeLookup.get(type);
    if (name == null) {
      return Strings2.lower(type.getSimpleName());
    }
    return name;
  }

  private AuditRecorder recorder() {
    checkState(auditRecorder != null, "Missing audit-recorder");
    return auditRecorder;
  }

  /**
   * @deprecated use {@link #isRecording()} instead
   */
  @Deprecated
  protected boolean isEnabled() {
    return isRecording();
  }

  /**
   * @since 3.2
   */
  protected boolean isRecording() {
    return recorder().isEnabled() && !EventHelper.isReplicating();
  }

  protected void record(final AuditData data) {
    recorder().record(data);
  }

  /**
   * Helper to convert an object into a string.
   */
  @Nullable
  protected static String string(final Object value) {
    if (value == null) {
      return null;
    }
    return String.valueOf(value);
  }

  /**
   * Helper to convert an iterable into a string.
   */
  protected static String string(final Iterable value) {
    if (value == null) {
      return null;
    }
    return LIST_JOINER.join(value);
  }
}
