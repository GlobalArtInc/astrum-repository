package org.globalart.astrum.repository.conan.internal;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.conan.AssetKind;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

public enum ConanEndpoint
{
  PING_V1(0, Methods.READ, "/v1/ping", null),
  PING_V2(0, Methods.READ, "/v2/ping", null),
  AUTHENTICATE_V1(0, Methods.READ, "/v1/users/authenticate", null),
  AUTHENTICATE_V2(0, Methods.READ, "/v2/users/authenticate", null),
  CHECK_CREDENTIALS_V1(0, Methods.READ, "/v1/users/check_credentials", null),
  CHECK_CREDENTIALS_V2(0, Methods.READ, "/v2/users/check_credentials", null),

  SEARCH_V2(2, Methods.READ, "/v2/conans/search", null),
  RECIPE_LATEST(2, Methods.READ, Patterns.V2 + "/latest", AssetKind.RECIPE_LATEST),
  RECIPE_REVISIONS(2, Methods.READ, Patterns.V2 + "/revisions", AssetKind.RECIPE_REVISIONS),
  RECIPE_SEARCH(2, Methods.READ, Patterns.V2 + "/search", AssetKind.PACKAGES_SEARCH),
  RECIPE_DELETE(2, Methods.REMOVE, Patterns.V2, null),
  RECIPE_REVISION_DELETE(2, Methods.REMOVE, Patterns.V2_REV, null),
  RECIPE_FILES(2, Methods.READ, Patterns.V2_REV + "/files", AssetKind.RECIPE_FILES),
  RECIPE_FILE(2, Methods.READ, Patterns.V2_REV + "/files/{path:.+}", AssetKind.RECIPE_FILE),
  RECIPE_FILE_UPLOAD(2, Methods.WRITE, Patterns.V2_REV + "/files/{path:.+}", AssetKind.RECIPE_FILE),
  PACKAGES_SEARCH(2, Methods.READ, Patterns.V2_REV + "/search", AssetKind.PACKAGES_SEARCH),
  PACKAGES_DELETE(2, Methods.REMOVE, Patterns.V2_REV + "/packages", null),
  PACKAGE_DELETE(2, Methods.REMOVE, Patterns.V2_PKG, null),
  PACKAGE_LATEST(2, Methods.READ, Patterns.V2_PKG + "/latest", AssetKind.PACKAGE_LATEST),
  PACKAGE_REVISIONS(2, Methods.READ, Patterns.V2_PKG + "/revisions", AssetKind.PACKAGE_REVISIONS),
  PACKAGE_REVISION_DELETE(2, Methods.REMOVE, Patterns.V2_PKG_REV, null),
  PACKAGE_FILES(2, Methods.READ, Patterns.V2_PKG_REV + "/files", AssetKind.PACKAGE_FILES),
  PACKAGE_FILE(2, Methods.READ, Patterns.V2_PKG_REV + "/files/{path:.+}", AssetKind.PACKAGE_FILE),
  PACKAGE_FILE_UPLOAD(2, Methods.WRITE, Patterns.V2_PKG_REV + "/files/{path:.+}", AssetKind.PACKAGE_FILE),

  SEARCH_V1(1, Methods.READ, "/v1/conans/search", null),
  V1_RECIPE_SNAPSHOT(1, Methods.READ, Patterns.V1, AssetKind.SNAPSHOT),
  V1_RECIPE_DELETE(1, Methods.REMOVE, Patterns.V1, null),
  V1_RECIPE_DIGEST(1, Methods.READ, Patterns.V1 + "/digest", AssetKind.DIGEST),
  V1_RECIPE_DOWNLOAD_URLS(1, Methods.READ, Patterns.V1 + "/download_urls", AssetKind.DOWNLOAD_URLS),
  V1_RECIPE_UPLOAD_URLS(1, Methods.CREATE, Patterns.V1 + "/upload_urls", null),
  V1_PACKAGES_SEARCH(1, Methods.READ, Patterns.V1 + "/search", AssetKind.PACKAGES_SEARCH),
  V1_PACKAGES_DELETE(1, Methods.CREATE, Patterns.V1 + "/packages/delete", null),
  V1_PACKAGE_SNAPSHOT(1, Methods.READ, Patterns.V1_PKG, AssetKind.SNAPSHOT),
  V1_PACKAGE_DIGEST(1, Methods.READ, Patterns.V1_PKG + "/digest", AssetKind.DIGEST),
  V1_PACKAGE_DOWNLOAD_URLS(1, Methods.READ, Patterns.V1_PKG + "/download_urls", AssetKind.DOWNLOAD_URLS),
  V1_PACKAGE_UPLOAD_URLS(1, Methods.CREATE, Patterns.V1_PKG + "/upload_urls", null),
  V1_RECIPE_FILE(1, Methods.READ, Patterns.V1_FILES + "/export/{path:.+}", AssetKind.RECIPE_FILE),
  V1_RECIPE_FILE_UPLOAD(1, Methods.WRITE, Patterns.V1_FILES + "/export/{path:.+}", AssetKind.RECIPE_FILE),
  V1_PACKAGE_FILE(1, Methods.READ, Patterns.V1_FILES + "/package/{pkgid}/{prev}/{path:.+}", AssetKind.PACKAGE_FILE),
  V1_PACKAGE_FILE_UPLOAD(1, Methods.WRITE, Patterns.V1_FILES + "/package/{pkgid}/{prev}/{path:.+}",
      AssetKind.PACKAGE_FILE);

  private final int protocol;

  private final String[] methods;

  private final String pattern;

  private final AssetKind assetKind;

  ConanEndpoint(final int protocol, final String[] methods, final String pattern, @Nullable final AssetKind assetKind) {
    this.protocol = protocol;
    this.methods = methods;
    this.pattern = pattern;
    this.assetKind = assetKind;
  }

  public int protocol() {
    return protocol;
  }

  @Nullable
  public AssetKind assetKind() {
    return assetKind;
  }

  public boolean isRead() {
    return methods == Methods.READ;
  }

  public Matcher matcher() {
    ConanEndpoint endpoint = this;
    return LogicMatchers.and(new ActionMatcher(methods), new TokenMatcher(pattern), context -> {
      context.getAttributes().set(ConanEndpoint.class, endpoint);
      if (endpoint.assetKind != null) {
        context.getAttributes().set(AssetKind.class, endpoint.assetKind);
      }
      return true;
    });
  }

  public static ConanEndpoint of(final Context context) {
    return context.getAttributes().require(ConanEndpoint.class);
  }

  private static final class Patterns
  {
    static final String REF = "/{name}/{version}/{user}/{channel}";

    static final String V2 = "/v2/conans" + REF;

    static final String V2_REV = V2 + "/revisions/{rrev}";

    static final String V2_PKG = V2_REV + "/packages/{pkgid}";

    static final String V2_PKG_REV = V2_PKG + "/revisions/{prev}";

    static final String V1 = "/v1/conans" + REF;

    static final String V1_PKG = V1 + "/packages/{pkgid}";

    static final String V1_FILES = "/v1/files" + REF + "/{rrev}";
  }

  private static final class Methods
  {
    static final String[] READ = {GET, HEAD};

    static final String[] WRITE = {PUT};

    static final String[] CREATE = {POST};

    static final String[] REMOVE = {DELETE};
  }
}
