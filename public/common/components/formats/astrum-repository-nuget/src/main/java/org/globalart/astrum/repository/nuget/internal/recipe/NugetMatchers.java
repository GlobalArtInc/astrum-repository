package org.globalart.astrum.repository.nuget.internal.recipe;

import java.util.regex.Pattern;

import org.globalart.astrum.repository.nuget.internal.AssetKind;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.LiteralMatcher;
import org.globalart.astrum.repository.view.matchers.RegexMatcher;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;
import static org.globalart.astrum.repository.view.matchers.logic.LogicMatchers.and;
import static org.globalart.astrum.repository.view.matchers.logic.LogicMatchers.or;

public final class NugetMatchers
{
  public static final String ID = "id";

  public static final String VERSION = "version";

  public static final String FILENAME = "filename";

  private NugetMatchers() {
  }

  public static Matcher kind(final Matcher matcher, final AssetKind kind) {
    return and(matcher, context -> {
      context.getAttributes().set(AssetKind.class, kind);
      return true;
    });
  }

  private static Matcher read(final Matcher matcher) {
    return and(new ActionMatcher(GET, HEAD), matcher);
  }

  public static Matcher serviceIndex() {
    return kind(read(new LiteralMatcher("/index.json")), AssetKind.SERVICE_INDEX);
  }

  public static Matcher flatVersions() {
    return kind(read(new TokenMatcher("/v3/content/{id}/index.json")), AssetKind.PACKAGE_VERSIONS);
  }

  public static Matcher flatPackage() {
    return kind(read(new TokenMatcher("/v3/content/{id}/{version}/{filename:.+\\.nupkg}")), AssetKind.PACKAGE);
  }

  public static Matcher flatNuspec() {
    return kind(read(new TokenMatcher("/v3/content/{id}/{version}/{filename:.+\\.nuspec}")), AssetKind.NUSPEC);
  }

  public static Matcher flatFile() {
    return kind(read(new TokenMatcher("/v3/content/{id}/{version}/{filename}")), AssetKind.PACKAGE_FILE);
  }

  public static Matcher assetDownload() {
    return kind(read(new TokenMatcher("/{id}/{version}/{filename:.+\\.[nN][uU][pP][kK][gG]}")), AssetKind.PACKAGE);
  }

  public static Matcher registrationIndex() {
    return kind(read(new TokenMatcher("/v3/registration/{id}/index.json")), AssetKind.REGISTRATION);
  }

  public static Matcher registrationPage() {
    return kind(read(new TokenMatcher("/v3/registration/{id}/page/{lower}/{upper}.json")), AssetKind.REGISTRATION);
  }

  public static Matcher registrationLeaf() {
    return kind(read(new TokenMatcher("/v3/registration/{id}/{version}.json")), AssetKind.REGISTRATION);
  }

  public static Matcher catalogEntry() {
    return kind(read(new TokenMatcher("/v3/catalog/{id}/{version}.json")), AssetKind.REGISTRATION);
  }

  public static Matcher query() {
    return read(new LiteralMatcher("/v3/query"));
  }

  public static Matcher autocomplete() {
    return read(new LiteralMatcher("/v3/autocomplete"));
  }

  public static Matcher push() {
    return and(new ActionMatcher(PUT, POST),
        or(new LiteralMatcher("/"), new LiteralMatcher("/api/v2/package"), new LiteralMatcher("/api/v2/package/")));
  }

  public static Matcher delete() {
    return and(new ActionMatcher(DELETE),
        or(new TokenMatcher("/api/v2/package/{id}/{version}"), new TokenMatcher("/{id}/{version}")));
  }

  public static Matcher v2Root() {
    return and(read(new LiteralMatcher("/")), context -> {
      String accept = context.getRequest().getHeaders().get("Accept");
      return accept == null || !accept.contains("text/html");
    });
  }

  public static Matcher v2Metadata() {
    return read(new LiteralMatcher("/$metadata"));
  }

  public static Matcher v2FindPackagesById() {
    return read(new RegexMatcher("/FindPackagesById(\\(\\))?(/\\$count)?", Pattern.CASE_INSENSITIVE));
  }

  public static Matcher v2PackagesEntry() {
    return read(new RegexMatcher("/Packages\\(Id='([^']+)',Version='([^']+)'\\)", Pattern.CASE_INSENSITIVE));
  }

  public static Matcher v2Packages() {
    return read(new RegexMatcher("/Packages(\\(\\))?(/\\$count)?", Pattern.CASE_INSENSITIVE));
  }

  public static Matcher v2Search() {
    return read(new RegexMatcher("/Search(\\(\\))?(/\\$count)?", Pattern.CASE_INSENSITIVE));
  }

  public static Matcher v2Download() {
    return kind(read(or(new TokenMatcher("/api/v2/package/{id}/{version}"), new TokenMatcher("/package/{id}/{version}"))),
        AssetKind.PACKAGE);
  }

  public static Matcher anyRead() {
    return read(context -> true);
  }
}
