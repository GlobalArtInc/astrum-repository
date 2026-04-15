
package org.globalart.astrum.repository.golang.internal.util;

import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.golang.internal.metadata.GolangAttributes;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import static com.google.common.base.Preconditions.checkNotNull;

public final class GolangPathUtils
{
  private static final Pattern VERSION = Pattern.compile(
      "v(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(-[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?(\\+incompatible)?");

  private static final Pattern MODULE_PATH = Pattern.compile("[A-Za-z0-9._~/-]+");

  private GolangPathUtils() {
  }

  public static String module(final TokenMatcher.State state) {
    return match(state, "module");
  }

  public static String version(final TokenMatcher.State state) {
    return match(state, "version");
  }

  public static String extension(final TokenMatcher.State state) {
    return match(state, "extension");
  }

  public static String assetPath(final TokenMatcher.State state) {
    return assetPath(module(state), version(state), extension(state));
  }

  public static String assetPath(final String module, final String version, final String extension) {
    return String.format("/%s/@v/%s.%s", module, version, extension);
  }

  public static String listPath(final TokenMatcher.State state) {
    return listPath(module(state));
  }

  public static String listPath(final String module) {
    return String.format("/%s/@v/list", module);
  }

  public static String latestPath(final TokenMatcher.State state) {
    return String.format("/%s/@latest", module(state));
  }

  public static TokenMatcher.State matcherState(final Context context) {
    return context.getAttributes().require(TokenMatcher.State.class);
  }

  public static GolangAttributes getAttributesFromMatcherState(final TokenMatcher.State state) {
    String module = unescape(module(state));
    String version = unescape(version(state));
    if (module == null || version == null) {
      throw new InvalidContentException("Invalid module path or version: " + module(state) + "@" + version(state));
    }
    GolangAttributes golangAttributes = new GolangAttributes();
    golangAttributes.setModule(module);
    golangAttributes.setVersion(version);
    return golangAttributes;
  }

  public static void validate(final GolangAttributes attributes) {
    String module = attributes.getModule();
    if (module == null || !MODULE_PATH.matcher(module).matches() || module.startsWith("/") || module.endsWith("/")
        || module.contains("//")) {
      throw new InvalidContentException("Invalid module path: " + module);
    }
    if (attributes.getVersion() == null || !VERSION.matcher(attributes.getVersion()).matches()) {
      throw new InvalidContentException("Invalid module version: " + attributes.getVersion());
    }
  }

  public static String escape(final String value) {
    StringBuilder escaped = new StringBuilder(value.length() + 8);
    for (char c : value.toCharArray()) {
      if (c >= 'A' && c <= 'Z') {
        escaped.append('!').append(Character.toLowerCase(c));
      }
      else {
        escaped.append(c);
      }
    }
    return escaped.toString();
  }

  @Nullable
  public static String unescape(final String escaped) {
    StringBuilder value = new StringBuilder(escaped.length());
    boolean bang = false;
    for (char c : escaped.toCharArray()) {
      if (bang) {
        if (c < 'a' || c > 'z') {
          return null;
        }
        value.append(Character.toUpperCase(c));
        bang = false;
      }
      else if (c == '!') {
        bang = true;
      }
      else if (c >= 'A' && c <= 'Z') {
        return null;
      }
      else {
        value.append(c);
      }
    }
    return bang ? null : value.toString();
  }

  private static String match(final TokenMatcher.State state, final String name) {
    checkNotNull(state);
    String result = state.getTokens().get(name);
    checkNotNull(result);
    return result;
  }
}
