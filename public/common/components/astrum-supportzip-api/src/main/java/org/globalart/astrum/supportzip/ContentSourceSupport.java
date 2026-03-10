
package org.globalart.astrum.supportzip;

import org.globalart.astrum.supportzip.SupportBundle.ContentSource;

import com.google.common.primitives.Ints;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for {@link ContentSource} implementations.
 *
 * @since 2.7
 */
public abstract class ContentSourceSupport
    implements ContentSource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Type type;

  private final String path;

  private Priority priority = Priority.DEFAULT;

  /**
   * @since 3.0
   */
  public ContentSourceSupport(final Type type, final String path, final Priority priority) {
    this.type = checkNotNull(type);
    this.path = normalize(checkNotNull(path));
    setPriority(priority);
  }

  public ContentSourceSupport(final Type type, final String path) {
    this(type, path, Priority.DEFAULT);
  }

  @Override
  public Type getType() {
    return type;
  }

  @Override
  public String getPath() {
    return path;
  }

  public Priority getPriority() {
    return priority;
  }

  public void setPriority(final Priority priority) {
    this.priority = checkNotNull(priority);
  }

  /**
   * Compare by priority order.
   */
  @Override
  public int compareTo(final ContentSource obj) {
    return Ints.compare(priority.order, obj.getPriority().order);
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "type=" + type +
        ", path='" + path + '\'' +
        ", priority=" + priority +
        '}';
  }

  private String normalize(final String path) {
    return path.replace("\\", "/");
  }
}
