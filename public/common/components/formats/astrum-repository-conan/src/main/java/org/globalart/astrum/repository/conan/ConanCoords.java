
package org.globalart.astrum.repository.conan;

import java.util.Map;
import java.util.Objects;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import static com.google.common.base.Preconditions.checkNotNull;

public final class ConanCoords
{
  public static final String NONE = "_";

  public static final String NAME = "name";

  public static final String VERSION = "version";

  public static final String USER = "user";

  public static final String CHANNEL = "channel";

  public static final String RECIPE_REVISION = "rrev";

  public static final String PACKAGE_ID = "pkgid";

  public static final String PACKAGE_REVISION = "prev";

  public static final String PATH = "path";

  private static final Pattern SEGMENT = Pattern.compile("[A-Za-z0-9_][A-Za-z0-9_+.\\-]*");

  private final String name;

  private final String version;

  private final String user;

  private final String channel;

  private final String recipeRevision;

  private final String packageId;

  private final String packageRevision;

  private final String file;

  public ConanCoords(
      final String name,
      final String version,
      @Nullable final String user,
      @Nullable final String channel,
      @Nullable final String recipeRevision,
      @Nullable final String packageId,
      @Nullable final String packageRevision,
      @Nullable final String file)
  {
    this.name = checkNotNull(name);
    this.version = checkNotNull(version);
    this.user = user == null || user.isEmpty() ? NONE : user;
    this.channel = channel == null || channel.isEmpty() ? NONE : channel;
    this.recipeRevision = recipeRevision;
    this.packageId = packageId;
    this.packageRevision = packageRevision;
    this.file = file;
  }

  public static ConanCoords fromState(final TokenMatcher.State state) {
    Map<String, String> tokens = state.getTokens();
    ConanCoords coords = new ConanCoords(tokens.get(NAME), tokens.get(VERSION), tokens.get(USER), tokens.get(CHANNEL),
        tokens.get(RECIPE_REVISION), tokens.get(PACKAGE_ID), tokens.get(PACKAGE_REVISION), tokens.get(PATH));
    coords.validate();
    return coords;
  }

  public void validate() {
    for (String segment : new String[]{name, version, user, channel, recipeRevision, packageId, packageRevision}) {
      if (segment != null && (!SEGMENT.matcher(segment).matches() || segment.contains(".."))) {
        throw new InvalidContentException("Invalid Conan coordinate: " + segment);
      }
    }
    if (file != null) {
      if (file.isEmpty() || file.startsWith("/") || file.endsWith("/") || file.contains("\\")) {
        throw new InvalidContentException("Invalid Conan file path: " + file);
      }
      for (String part : file.split("/")) {
        if (part.isEmpty() || part.equals(".") || part.equals("..")) {
          throw new InvalidContentException("Invalid Conan file path: " + file);
        }
      }
    }
  }

  public String getName() {
    return name;
  }

  public String getVersion() {
    return version;
  }

  public String getUser() {
    return user;
  }

  public String getChannel() {
    return channel;
  }

  @Nullable
  public String getRecipeRevision() {
    return recipeRevision;
  }

  @Nullable
  public String getPackageId() {
    return packageId;
  }

  @Nullable
  public String getPackageRevision() {
    return packageRevision;
  }

  @Nullable
  public String getFile() {
    return file;
  }

  public boolean hasUserChannel() {
    return !NONE.equals(user) || !NONE.equals(channel);
  }

  public String namespace() {
    if (!hasUserChannel()) {
      return "";
    }
    return NONE.equals(channel) ? user : user + "/" + channel;
  }

  public String reference() {
    return name + "/" + version + "@" + user + "/" + channel;
  }

  public ConanCoords withRecipeRevision(@Nullable final String revision) {
    return new ConanCoords(name, version, user, channel, revision, packageId, packageRevision, file);
  }

  public ConanCoords withPackage(@Nullable final String id, @Nullable final String revision) {
    return new ConanCoords(name, version, user, channel, recipeRevision, id, revision, file);
  }

  public ConanCoords withPackageRevision(@Nullable final String revision) {
    return new ConanCoords(name, version, user, channel, recipeRevision, packageId, revision, file);
  }

  public ConanCoords withFile(@Nullable final String path) {
    return new ConanCoords(name, version, user, channel, recipeRevision, packageId, packageRevision, path);
  }

  public ConanCoords recipe() {
    return new ConanCoords(name, version, user, channel, null, null, null, null);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof ConanCoords that)) {
      return false;
    }
    return name.equals(that.name) && version.equals(that.version) && user.equals(that.user) &&
        channel.equals(that.channel) && Objects.equals(recipeRevision, that.recipeRevision) &&
        Objects.equals(packageId, that.packageId) && Objects.equals(packageRevision, that.packageRevision) &&
        Objects.equals(file, that.file);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, version, user, channel, recipeRevision, packageId, packageRevision, file);
  }

  @Override
  public String toString() {
    StringBuilder text = new StringBuilder(reference());
    if (recipeRevision != null) {
      text.append('#').append(recipeRevision);
    }
    if (packageId != null) {
      text.append(':').append(packageId);
    }
    if (packageRevision != null) {
      text.append('#').append(packageRevision);
    }
    if (file != null) {
      text.append(' ').append(file);
    }
    return text.toString();
  }
}
