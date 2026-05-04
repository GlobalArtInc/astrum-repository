package org.globalart.astrum.repository.cargo.internal;

import java.util.Map;

import org.globalart.astrum.repository.cargo.AssetKind;
import org.globalart.astrum.repository.cargo.CrateCoordinates;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.LiteralMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

public final class CargoPaths
{
  public static final String CONFIG = "/config.json";

  public static final String ME = "/me";

  public static final String CRATES_API = "/api/v1/crates";

  public static final String PUBLISH = CRATES_API + "/new";

  public static final String YANK_SUFFIX = "/yank";

  public static final String UNYANK_SUFFIX = "/unyank";

  public static final String OWNERS_SUFFIX = "/owners";

  public static final String INDEX_NAME = "cargo.index.name";

  private CargoPaths() {
  }

  public static Matcher config() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher(CONFIG));
  }

  public static Matcher me() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD, DELETE), new LiteralMatcher(ME));
  }

  public static Matcher search() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher(CRATES_API));
  }

  public static Matcher publish() {
    return LogicMatchers.and(new ActionMatcher(PUT), new LiteralMatcher(PUBLISH));
  }

  public static Matcher yank() {
    return LogicMatchers.and(new ActionMatcher(DELETE),
        crateTokens(new TokenMatcher(CRATES_API + "/{name:[^/]+}/{version:[^/]+}" + YANK_SUFFIX)));
  }

  public static Matcher unyank() {
    return LogicMatchers.and(new ActionMatcher(PUT),
        crateTokens(new TokenMatcher(CRATES_API + "/{name:[^/]+}/{version:[^/]+}" + UNYANK_SUFFIX)));
  }

  public static Matcher owners() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD, PUT, DELETE),
        new TokenMatcher(CRATES_API + "/{name:[^/]+}" + OWNERS_SUFFIX), context -> {
          String name = tokens(context).get("name");
          if (!CrateCoordinates.isValidName(name)) {
            return false;
          }
          context.getAttributes().set(INDEX_NAME, name);
          return true;
        });
  }

  public static Matcher download() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD),
        LogicMatchers.or(
            crateTokens(new TokenMatcher(CRATES_API + "/{name:[^/]+}/{version:[^/]+}/download")),
            crateFile()));
  }

  public static Matcher index() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), context -> {
      String name = CrateCoordinates.nameFromIndexPath(context.getRequest().getPath());
      if (name == null) {
        return false;
      }
      context.getAttributes().set(INDEX_NAME, name);
      context.getAttributes().set(AssetKind.class, AssetKind.INDEX);
      return true;
    });
  }

  private static Matcher crateFile() {
    return LogicMatchers.and(
        new TokenMatcher(CrateCoordinates.CRATES_PREFIX + "{name:[^/]+}/{version:[^/]+}/{file:[^/]+}"),
        context -> {
          Map<String, String> tokens = tokens(context);
          String expected = tokens.get("name") + "-" + tokens.get("version") + CrateCoordinates.CRATE_EXTENSION;
          return expected.equals(tokens.get("file")) && setCoordinates(context, tokens);
        });
  }

  private static Matcher crateTokens(final TokenMatcher tokenMatcher) {
    return LogicMatchers.and(tokenMatcher, context -> setCoordinates(context, tokens(context)));
  }

  private static boolean setCoordinates(final Context context, final Map<String, String> tokens) {
    String name = tokens.get("name");
    String version = tokens.get("version");
    if (!CrateCoordinates.isValidName(name) || !CrateCoordinates.isValidVersion(version)) {
      return false;
    }
    context.getAttributes().set(CrateCoordinates.class, new CrateCoordinates(name, version));
    context.getAttributes().set(AssetKind.class, AssetKind.CRATE);
    return true;
  }

  private static Map<String, String> tokens(final Context context) {
    return context.getAttributes().require(TokenMatcher.State.class).getTokens();
  }

  public static CrateCoordinates coordinates(final Context context) {
    return context.getAttributes().require(CrateCoordinates.class);
  }

  public static String indexName(final Context context) {
    return context.getAttributes().require(INDEX_NAME, String.class);
  }
}
