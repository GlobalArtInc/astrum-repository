
package org.globalart.astrum.capability;

import java.util.Set;

import com.google.common.collect.Sets;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A Capability tag (key/value).
 *
 * @since 2.7
 */
public class Tag
{

  /**
   * Key of category tag.
   */
  public static final String CATEGORY = "Category";

  /**
   * Key of repository tag.
   */
  public static final String REPOSITORY = "Repository";

  private final String key;

  private final String value;

  public Tag(final String key, final String value) {
    this.key = checkNotNull(key);
    this.value = checkNotNull(value);
  }

  public String key() {
    return key;
  }

  public String value() {
    return value;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof Tag)) {
      return false;
    }

    Tag tag = (Tag) o;

    if (!key.equals(tag.key)) {
      return false;
    }

    return true;
  }

  @Override
  public int hashCode() {
    return key.hashCode();
  }

  /**
   * Convenience method for creating a set of tags
   */
  public static Set<Tag> tags(final Tag... tags) {
    return Sets.newHashSet(checkNotNull(tags));
  }

  /**
   * Convenience method for a category tag.
   */
  public static Tag categoryTag(final String category) {
    return new Tag(CATEGORY, category);
  }

  /**
   * Convenience method for a repository tag.
   */
  public static Tag repositoryTag(final String repository) {
    return new Tag(REPOSITORY, repository);
  }

}
