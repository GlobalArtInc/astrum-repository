

package org.globalart.astrum.content.maven.store;

import com.google.common.base.Objects;

/**
 *
 * Struct to track GAV we need to request metadata rebuild due to deletion.
 *
 * @since 3.30
 */
public class GAV
{
  public final String group;

  public final String name;

  public final String baseVersion;

  final int count;

  public GAV(final String group, final String name, final String baseVersion, final int count) {
    this.group = group;
    this.name = name;
    this.baseVersion = baseVersion;
    this.count = count;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    GAV gav = (GAV) o;
    return count == gav.count && Objects.equal(group, gav.group) && Objects.equal(name, gav.name) &&
        Objects.equal(baseVersion, gav.baseVersion);
  }

  @Override
  public int hashCode() {
    return Objects.hashCode(group, name, baseVersion, count);
  }

  @Override
  public String toString() {
    return "GAV{" + "group='" + group + '\'' + ", name='" + name + '\'' + ", baseVersion='" + baseVersion + '\'' +
        ", count=" + count + '}';
  }
}
