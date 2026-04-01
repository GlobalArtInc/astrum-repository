
package org.globalart.astrum.repository.pypi.internal;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import com.google.common.base.Objects;

import static com.google.common.base.Preconditions.checkNotNull;

public final class PyPiLink
{
  private final String file;

  private final String link;

  private final String dataRequiresPython;

  @Nullable
  private final String yanked;

  public PyPiLink(
      @Nonnull final String file,
      @Nonnull final String link,
      @Nullable final String dataRequiresPython,
      @Nullable final String yanked)
  {
    this.file = checkNotNull(file);
    this.link = checkNotNull(link);
    this.dataRequiresPython = dataRequiresPython != null ? dataRequiresPython : "";
    this.yanked = yanked;
  }

  public PyPiLink(@Nonnull final String file, @Nonnull final String link, @Nullable final String dataRequiresPython) {
    this(file, link, dataRequiresPython, null);
  }

  public PyPiLink(final String file, final String link) {
    this(file, link, "", null);
  }

  public String getLink() {
    return link;
  }

  public String getFile() {
    return file;
  }

  public String getDataRequiresPython() {
    return dataRequiresPython;
  }

  @Nullable
  public String getYanked() {
    return yanked;
  }

  public PyPiLink withLink(final String newLink) {
    return new PyPiLink(file, newLink, dataRequiresPython, yanked);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    PyPiLink pyPiLink = (PyPiLink) o;
    return Objects.equal(file, pyPiLink.file) &&
        Objects.equal(link, pyPiLink.link) &&
        Objects.equal(dataRequiresPython, pyPiLink.dataRequiresPython) &&
        Objects.equal(yanked, pyPiLink.yanked);
  }

  @Override
  public int hashCode() {
    return Objects.hashCode(file, link, dataRequiresPython, yanked);
  }
}
