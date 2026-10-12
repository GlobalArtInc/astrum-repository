package org.globalart.astrum.repository.rubygems.internal;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.rubygems.AssetKind;
import org.globalart.astrum.repository.rubygems.internal.gem.GemCoordinates;
import org.globalart.astrum.repository.rubygems.internal.index.LegacySpecs;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.LiteralMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;

import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

public final class RubygemsPaths
{
  public static final String VERSIONS = "/versions";

  public static final String NAMES = "/names";

  public static final String INFO_PREFIX = "/info/";

  public static final String DEPENDENCIES = "/api/v1/dependencies";

  public static final String GEMS_API = "/api/v1/gems";

  public static final String YANK = GEMS_API + "/yank";

  public static final String API_KEY = "/api/v1/api_key";

  public static final String GEM_NAME = "rubygems.name";

  public static final String FULL_NAME = "rubygems.fullName";

  private RubygemsPaths() {
  }

  public static Matcher versions() {
    return indexFile(VERSIONS);
  }

  public static Matcher names() {
    return indexFile(NAMES);
  }

  public static Matcher legacySpecs() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      if (LegacySpecs.Kind.fromPath(context.getRequest().getPath()) == null) {
        return false;
      }
      context.getAttributes().set(AssetKind.class, AssetKind.INDEX);
      return true;
    });
  }

  public static Matcher info() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      String path = context.getRequest().getPath();
      if (!path.startsWith(INFO_PREFIX)) {
        return false;
      }
      String name = path.substring(INFO_PREFIX.length());
      if (!GemCoordinates.isValidName(name)) {
        return false;
      }
      context.getAttributes().set(GEM_NAME, name);
      context.getAttributes().set(AssetKind.class, AssetKind.INDEX);
      return true;
    });
  }

  public static Matcher quickSpec() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      String fullName = fileName(context, GemCoordinates.QUICK_PREFIX, GemCoordinates.QUICK_EXTENSION);
      if (fullName == null) {
        return false;
      }
      context.getAttributes().set(FULL_NAME, fullName);
      context.getAttributes().set(AssetKind.class, AssetKind.GEMSPEC);
      return true;
    });
  }

  public static Matcher gem() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      String fullName = fileName(context, GemCoordinates.GEMS_PREFIX, GemCoordinates.GEM_EXTENSION);
      if (fullName == null) {
        return false;
      }
      context.getAttributes().set(FULL_NAME, fullName);
      context.getAttributes().set(AssetKind.class, AssetKind.GEM);
      return true;
    });
  }

  public static Matcher dependencies() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher(DEPENDENCIES));
  }

  public static Matcher push() {
    return LogicMatchers.and(new ActionMatcher(POST), new LiteralMatcher(GEMS_API));
  }

  public static Matcher yank() {
    return LogicMatchers.and(new ActionMatcher(DELETE), new LiteralMatcher(YANK));
  }

  public static Matcher apiKey() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD, POST, PUT, DELETE), new LiteralMatcher(API_KEY));
  }

  public static String gemName(final Context context) {
    return context.getAttributes().require(GEM_NAME, String.class);
  }

  public static String fullName(final Context context) {
    return context.getAttributes().require(FULL_NAME, String.class);
  }

  @Nullable
  private static String fileName(final Context context, final String prefix, final String extension) {
    String path = context.getRequest().getPath();
    if (!path.startsWith(prefix) || !path.endsWith(extension)) {
      return null;
    }
    String fileName = path.substring(prefix.length());
    if (!GemCoordinates.isValidFileName(fileName)) {
      return null;
    }
    String fullName = fileName.substring(0, fileName.length() - extension.length());
    return fullName.isEmpty() ? null : fullName;
  }

  private static Matcher indexFile(final String path) {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher(path), context -> {
      context.getAttributes().set(AssetKind.class, AssetKind.INDEX);
      return true;
    });
  }
}
