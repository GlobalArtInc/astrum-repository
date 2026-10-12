package org.globalart.astrum.repository.terraform;

import java.util.Locale;
import java.util.Optional;
import java.util.function.BiFunction;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;

import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

/**
 * Paths of a Terraform repository and the matchers that route them.
 *
 * <pre>
 * /.well-known/terraform.json                                  service discovery
 * /token                                                       bearer token of the current user
 * /v1/modules/{namespace}/{name}/{system}/versions             module registry protocol
 * /v1/modules/{namespace}/{name}/{system}/{version}/download
 * /v1/modules/{namespace}/{name}/{system}/{version}/{archive}  module package
 * /v1/providers/{namespace}/{type}/versions                    provider registry protocol
 * /v1/providers/{namespace}/{type}/{version}/download/{os}/{arch}
 * /v1/providers/{namespace}/{type}/{version}/{file}            provider release files
 * /{hostname}/{namespace}/{type}/index.json                    provider network mirror protocol
 * /{hostname}/{namespace}/{type}/{version}.json
 * </pre>
 */
public final class TerraformPaths
{
  public static final String DISCOVERY = "/.well-known/terraform.json";

  public static final String TOKEN = "/token";

  public static final String MODULES = "/v1/modules/";

  public static final String PROVIDERS = "/v1/providers/";

  public static final String MODULES_SERVICE = "modules.v1";

  public static final String PROVIDERS_SERVICE = "providers.v1";

  private static final String SEGMENT = "([^/]+)";

  private static final Pattern NAME = Pattern.compile("[A-Za-z0-9](?:[A-Za-z0-9_-]{0,126}[A-Za-z0-9])?");

  private static final Pattern VERSION =
      Pattern.compile("[0-9]+\\.[0-9]+\\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\\+[0-9A-Za-z.-]+)?");

  private static final Pattern PLATFORM_PART = Pattern.compile("[a-z0-9]{1,32}");

  private static final Pattern HOSTNAME =
      Pattern.compile("[A-Za-z0-9](?:[A-Za-z0-9.-]{0,251}[A-Za-z0-9])?(?::[0-9]{1,5})?");

  private static final Pattern MODULE_VERSIONS = Pattern.compile("/v1/modules/" + SEGMENT + "/" + SEGMENT + "/"
      + SEGMENT + "/versions");

  private static final Pattern MODULE_DOWNLOAD = Pattern.compile("/v1/modules/" + SEGMENT + "/" + SEGMENT + "/"
      + SEGMENT + "/" + SEGMENT + "/download");

  private static final Pattern MODULE_FILE = Pattern.compile("/v1/modules/" + SEGMENT + "/" + SEGMENT + "/"
      + SEGMENT + "/" + SEGMENT + "/" + SEGMENT);

  private static final Pattern PROVIDER_VERSIONS = Pattern.compile("/v1/providers/" + SEGMENT + "/" + SEGMENT
      + "/versions");

  private static final Pattern PROVIDER_DOWNLOAD = Pattern.compile("/v1/providers/" + SEGMENT + "/" + SEGMENT + "/"
      + SEGMENT + "/download/" + SEGMENT + "/" + SEGMENT);

  private static final Pattern PROVIDER_FILE = Pattern.compile("/v1/providers/" + SEGMENT + "/" + SEGMENT + "/"
      + SEGMENT + "/" + SEGMENT);

  private static final Pattern MIRROR_INDEX = Pattern.compile("/" + SEGMENT + "/" + SEGMENT + "/" + SEGMENT
      + "/index\\.json");

  private static final Pattern MIRROR_VERSION = Pattern.compile("/" + SEGMENT + "/" + SEGMENT + "/" + SEGMENT
      + "/" + SEGMENT + "\\.json");

  private TerraformPaths() {
  }

  public static boolean isValidName(@Nullable final String value) {
    return value != null && NAME.matcher(value).matches();
  }

  public static boolean isValidVersion(@Nullable final String value) {
    return value != null && VERSION.matcher(value).matches();
  }

  public static boolean isValidPlatformPart(@Nullable final String value) {
    return value != null && PLATFORM_PART.matcher(value).matches();
  }

  public static boolean isValidHostname(@Nullable final String value) {
    return value != null && HOSTNAME.matcher(value).matches();
  }

  public static Matcher discovery() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), literal(DISCOVERY, TerraformRequest.Endpoint.DISCOVERY));
  }

  public static Matcher token() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD, DELETE), literal(TOKEN, TerraformRequest.Endpoint.TOKEN));
  }

  public static Matcher moduleVersions() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), regex(MODULE_VERSIONS, (context, m) -> module(m, null)
        .map(coordinates -> TerraformRequest.moduleVersions(coordinates))));
  }

  public static Matcher moduleDownload() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), regex(MODULE_DOWNLOAD, (context, m) ->
        module(m, m.group(4)).map(TerraformRequest::moduleDownload)));
  }

  public static Matcher moduleArchive(final String... actions) {
    return LogicMatchers.and(new ActionMatcher(actions), regex(MODULE_FILE, (context, m) -> {
      String file = m.group(5);
      if ("download".equals(file) || ModuleArchives.extension(file).isEmpty()) {
        return Optional.empty();
      }
      return module(m, m.group(4)).map(coordinates -> TerraformRequest.moduleArchive(coordinates, file));
    }));
  }

  public static Matcher providerVersions() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), regex(PROVIDER_VERSIONS, (context, m) ->
        provider(m, null).map(TerraformRequest::providerVersions)));
  }

  public static Matcher providerDownload() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), regex(PROVIDER_DOWNLOAD, (context, m) -> {
      if (!isValidPlatformPart(m.group(4)) || !isValidPlatformPart(m.group(5))) {
        return Optional.empty();
      }
      Platform platform = new Platform(m.group(4), m.group(5));
      return provider(m, m.group(3)).map(coordinates -> TerraformRequest.providerDownload(coordinates, platform));
    }));
  }

  public static Matcher providerFile(final String... actions) {
    return LogicMatchers.and(new ActionMatcher(actions), regex(PROVIDER_FILE, (context, m) -> {
      String file = m.group(4);
      Optional<ProviderCoordinates> coordinates = provider(m, m.group(3));
      if (coordinates.isEmpty()) {
        return Optional.empty();
      }
      return ProviderFiles.kindOf(coordinates.get(), file)
          .map(kind -> TerraformRequest.providerFile(coordinates.get(), file, kind));
    }));
  }

  public static Matcher mirrorIndex() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), regex(MIRROR_INDEX, (context, m) -> {
      if (!isValidHostname(m.group(1))) {
        return Optional.empty();
      }
      return provider(m.group(2), m.group(3), null)
          .map(coordinates -> TerraformRequest.mirrorIndex(m.group(1).toLowerCase(Locale.ROOT), coordinates));
    }));
  }

  public static Matcher mirrorVersion() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), regex(MIRROR_VERSION, (context, m) -> {
      if (!isValidHostname(m.group(1)) || "index".equals(m.group(4))) {
        return Optional.empty();
      }
      return provider(m.group(2), m.group(3), m.group(4))
          .map(coordinates -> TerraformRequest.mirrorVersion(m.group(1).toLowerCase(Locale.ROOT), coordinates));
    }));
  }

  /**
   * Relative URL from a document at {@code fromPath} to {@code toPath}, both absolute paths inside the repository.
   * Clients resolve it against the URL they fetched the document from, which keeps it valid behind a reverse proxy
   * that maps the repository to another path.
   */
  public static String relative(final String fromPath, final String toPath) {
    String[] from = fromPath.substring(1).split("/", -1);
    String[] to = toPath.substring(1).split("/", -1);
    int common = 0;
    while (common < from.length - 1 && common < to.length - 1 && from[common].equals(to[common])) {
      common++;
    }
    StringBuilder relative = new StringBuilder();
    for (int i = common; i < from.length - 1; i++) {
      relative.append("../");
    }
    if (relative.length() == 0) {
      relative.append("./");
    }
    for (int i = common; i < to.length; i++) {
      relative.append(to[i]);
      if (i < to.length - 1) {
        relative.append('/');
      }
    }
    return relative.toString();
  }

  private static Optional<ModuleCoordinates> module(final java.util.regex.Matcher m, @Nullable final String version) {
    if (!isValidName(m.group(1)) || !isValidName(m.group(2)) || !isValidName(m.group(3))
        || (version != null && !isValidVersion(version))) {
      return Optional.empty();
    }
    return Optional.of(new ModuleCoordinates(m.group(1), m.group(2), m.group(3), version));
  }

  private static Optional<ProviderCoordinates> provider(
      final java.util.regex.Matcher m,
      @Nullable final String version)
  {
    return provider(m.group(1), m.group(2), version);
  }

  private static Optional<ProviderCoordinates> provider(
      final String namespace,
      final String type,
      @Nullable final String version)
  {
    if (!isValidName(namespace) || !isValidName(type) || (version != null && !isValidVersion(version))) {
      return Optional.empty();
    }
    return Optional.of(new ProviderCoordinates(namespace, type, version));
  }

  private static Matcher literal(final String path, final TerraformRequest.Endpoint endpoint) {
    return context -> {
      if (!path.equals(context.getRequest().getPath())) {
        return false;
      }
      context.getAttributes().set(TerraformRequest.class, TerraformRequest.of(endpoint));
      return true;
    };
  }

  private static Matcher regex(
      final Pattern pattern,
      final BiFunction<Context, java.util.regex.Matcher, Optional<TerraformRequest>> parser)
  {
    return context -> {
      java.util.regex.Matcher m = pattern.matcher(context.getRequest().getPath());
      if (!m.matches()) {
        return false;
      }
      Optional<TerraformRequest> request = parser.apply(context, m);
      request.ifPresent(value -> {
        context.getAttributes().set(TerraformRequest.class, value);
        if (value.assetKind() != null) {
          context.getAttributes().set(AssetKind.class, value.assetKind());
        }
      });
      return request.isPresent();
    };
  }
}
