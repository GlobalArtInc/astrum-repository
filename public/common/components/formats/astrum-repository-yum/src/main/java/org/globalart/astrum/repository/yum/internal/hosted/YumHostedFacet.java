package org.globalart.astrum.repository.yum.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashSet;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeSet;
import java.util.concurrent.locks.Lock;
import java.util.function.Consumer;

import javax.annotation.Nullable;

import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.ContentTypes;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.YumContentFacet;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.LayoutPolicy;
import org.globalart.astrum.repository.yum.internal.YumPaths;
import org.globalart.astrum.repository.yum.internal.YumSigningFacet;
import org.globalart.astrum.repository.yum.internal.repodata.GeneratedMetadata;
import org.globalart.astrum.repository.yum.internal.repodata.Repodata;
import org.globalart.astrum.repository.yum.internal.repodata.RepodataBuilder;
import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;
import org.globalart.astrum.repository.yum.internal.rpm.RpmParser;

import com.google.common.util.concurrent.Striped;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.yum.YumContentFacet.ATTRIBUTES_KEY;

@Facet.Exposed
@Component
@Qualifier(YumFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class YumHostedFacet
    extends FacetSupport
{
  public static final String CONFIG_KEY = "yum";

  public static final String REPODATA_DEPTH = "repodataDepth";

  public static final String DEPLOY_POLICY = "deployPolicy";

  public static final String FINGERPRINT = "fingerprint";

  public static final String PACKAGE_COUNT = "packageCount";

  public static final String FILES = "files";

  public static final String SIGNER = "signer";

  public static class Config
  {
    @NotNull
    @Min(0)
    @Max(5)
    public Integer repodataDepth = 0;

    public LayoutPolicy deployPolicy = LayoutPolicy.STRICT;
  }

  private final Striped<Lock> locks = Striped.lock(64);

  private Config config;

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    facet(ConfigurationFacet.class).validateSection(configuration, CONFIG_KEY, Config.class, Default.class,
        getRepository().getType().getValidationGroup());
  }

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    config = facet(ConfigurationFacet.class).readSection(configuration, CONFIG_KEY, Config.class);
    if (config.repodataDepth == null) {
      config.repodataDepth = 0;
    }
    if (config.deployPolicy == null) {
      config.deployPolicy = LayoutPolicy.STRICT;
    }
  }

  @Override
  protected void doDestroy() throws Exception {
    config = null;
  }

  public int repodataDepth() {
    return config.repodataDepth;
  }

  public LayoutPolicy layoutPolicy() {
    return config.deployPolicy;
  }

  public Content put(final String path, final Payload payload) throws IOException {
    checkWritable(path);
    YumContentFacet content = content();
    try (TempBlob blob = content.ingest(payload)) {
      if (YumPaths.isRpm(path)) {
        RpmPackage rpmPackage = parse(blob);
        checkDepth(path);
        return content.putRpm(path, blob, rpmPackage);
      }
      if (layoutPolicy() == LayoutPolicy.STRICT) {
        throw new InvalidContentException(
            "Only RPM packages can be stored in a repository with the strict layout policy: " + path);
      }
      return content.putOther(path, blob);
    }
  }

  public Content upload(
      final String directory,
      @Nullable final String fileName,
      final Payload payload,
      final Consumer<String> authorizer) throws IOException
  {
    YumContentFacet content = content();
    try (TempBlob blob = content.ingest(payload)) {
      RpmPackage rpmPackage = parse(blob);
      String name = fileName != null && YumPaths.isRpm(fileName) ? YumPaths.fileName(fileName) : rpmPackage.fileName();
      String path = YumPaths.normalize(directory + "/" + name);
      checkWritable(path);
      checkDepth(path);
      authorizer.accept(path);
      return content.putRpm(path, blob, rpmPackage);
    }
  }

  public Optional<Content> get(final String path) {
    return content().get(path);
  }

  public Optional<Content> getRepomd(final String base) throws IOException {
    Optional<Content> repomd = content().get(YumPaths.repomdPath(base));
    if (repomd.isPresent()) {
      return repomd;
    }
    if (YumPaths.depthOf(base) != repodataDepth()) {
      return Optional.empty();
    }
    return Optional.ofNullable(rebuild(base, packagesUnder(base), false));
  }

  public Optional<Content> getPublicKey() {
    return facet(YumSigningFacet.class).publicKey();
  }

  public boolean delete(final String path) {
    Optional<FluentAsset> asset = content().getAsset(path);
    if (asset.isEmpty() || isMetadata(asset.get())) {
      return false;
    }
    facet(ContentMaintenanceFacet.class).deleteAsset(asset.get());
    return true;
  }

  public void rebuildMetadata(final boolean force) throws IOException {
    Map<String, List<FluentAsset>> packages = new TreeMap<>();
    YumContentFacet content = content();
    int depth = repodataDepth();
    for (FluentAsset asset : content.assetsOfKind(AssetKind.RPM)) {
      YumPaths.base(asset.path(), depth)
          .ifPresent(base -> packages.computeIfAbsent(base, key -> new ArrayList<>()).add(asset));
    }
    Set<String> bases = new TreeSet<>(packages.keySet());
    if (depth == 0) {
      bases.add("");
    }
    for (FluentAsset repomd : content.assetsOfKind(AssetKind.REPOMD)) {
      YumPaths.baseOfMetadata(repomd.path()).ifPresent(bases::add);
    }
    IOException failure = null;
    for (String base : bases) {
      try {
        rebuild(base, packages.getOrDefault(base, List.of()), force);
      }
      catch (IOException e) {
        log.warn("Unable to rebuild {} of repository {}", YumPaths.repomdPath(base), getRepository().getName(), e);
        failure = e;
      }
    }
    if (failure != null) {
      throw failure;
    }
  }

  @Nullable
  Content rebuild(final String base, final List<FluentAsset> packages, final boolean force) throws IOException {
    Lock lock = locks.get(base);
    lock.lock();
    try {
      YumContentFacet content = content();
      YumSigningFacet signing = facet(YumSigningFacet.class);
      boolean valid = YumPaths.depthOf(base) == repodataDepth();
      if (!valid || (packages.isEmpty() && !base.isEmpty())) {
        removeMetadata(base);
        return null;
      }
      List<FluentAsset> sorted = new ArrayList<>(packages);
      sorted.sort(Comparator.comparing(FluentAsset::path));
      String fingerprint = fingerprint(sorted, signing.signerId());
      Optional<FluentAsset> existing = content.getAsset(YumPaths.repomdPath(base));
      if (!force && existing.isPresent()
          && fingerprint.equals(existing.get().attributes(ATTRIBUTES_KEY).get(FINGERPRINT, String.class))) {
        return existing.get().download();
      }
      List<AssetPackageSource> sources = sorted.stream().map(asset -> new AssetPackageSource(asset, base)).toList();
      long timestamp = Instant.now().getEpochSecond();
      try (Repodata repodata = RepodataBuilder.build(sources, timestamp)) {
        List<String> files = new ArrayList<>();
        for (GeneratedMetadata file : repodata.files()) {
          content.putMetadata(YumPaths.repodataDirectory(base) + "/" + file.fileName(), AssetKind.REPODATA,
              file.file(), ContentTypes.APPLICATION_GZIP, Map.of());
          files.add(file.fileName());
        }
        byte[] repomd = repodata.repomd().toXml();
        Optional<byte[]> signature = signing.sign(repomd);
        if (signature.isPresent()) {
          content.putMetadata(YumPaths.signaturePath(base), AssetKind.REPODATA, signature.get(),
              YumSigningFacet.SIGNATURE_CONTENT_TYPE, Map.of());
        }
        else {
          content.delete(YumPaths.signaturePath(base));
        }
        Set<String> keep = new HashSet<>(files);
        existing.ifPresent(asset -> keep.addAll(filesOf(asset)));
        Map<String, Object> attributes = new LinkedHashMap<>();
        attributes.put(FINGERPRINT, fingerprint);
        attributes.put(PACKAGE_COUNT, repodata.packages());
        attributes.put(FILES, files);
        attributes.put(SIGNER, signing.signerId());
        Content stored = content.putMetadata(YumPaths.repomdPath(base), AssetKind.REPOMD, repomd,
            ContentTypes.APPLICATION_XML, attributes);
        removeUnreferenced(base, keep, signature.isPresent());
        log.debug("Rebuilt {} of repository {} with {} packages", YumPaths.repomdPath(base),
            getRepository().getName(), repodata.packages());
        if (!repodata.skipped().isEmpty()) {
          log.warn("Left unreadable packages {} out of {} of repository {}", repodata.skipped(),
              YumPaths.repomdPath(base), getRepository().getName());
        }
        return stored;
      }
    }
    finally {
      lock.unlock();
    }
  }

  private List<FluentAsset> packagesUnder(final String base) {
    int depth = repodataDepth();
    return content().assetsUnder(base.isEmpty() ? "/" : base, AssetKind.RPM).stream()
        .filter(asset -> YumPaths.base(asset.path(), depth).map(base::equals).orElse(false))
        .toList();
  }

  private void removeMetadata(final String base) {
    YumContentFacet content = content();
    for (AssetKind kind : List.of(AssetKind.REPOMD, AssetKind.REPODATA)) {
      for (FluentAsset asset : content.assetsUnder(YumPaths.repodataDirectory(base), kind)) {
        if (YumPaths.directory(asset.path()).equals(YumPaths.repodataDirectory(base))) {
          asset.delete();
        }
      }
    }
  }

  private void removeUnreferenced(final String base, final Set<String> keep, final boolean signed) {
    String directory = YumPaths.repodataDirectory(base);
    for (FluentAsset asset : content().assetsUnder(directory, AssetKind.REPODATA)) {
      String name = YumPaths.fileName(asset.path());
      if (!YumPaths.directory(asset.path()).equals(directory) || keep.contains(name)
          || (signed && YumPaths.SIGNATURE.equals(name))) {
        continue;
      }
      asset.delete();
    }
  }

  @SuppressWarnings("unchecked")
  private static List<String> filesOf(final FluentAsset repomd) {
    Object files = repomd.attributes(ATTRIBUTES_KEY).get(FILES);
    return files instanceof List<?> list ? (List<String>) list : List.of();
  }

  static String fingerprint(final List<FluentAsset> packages, final String signerId) {
    MessageDigest digest;
    try {
      digest = MessageDigest.getInstance("SHA-256");
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
    digest.update(signerId.getBytes(StandardCharsets.UTF_8));
    for (FluentAsset asset : packages) {
      digest.update(("\n" + asset.path() + "\t" + AssetPackageSource.checksumOf(asset))
          .getBytes(StandardCharsets.UTF_8));
    }
    return HexFormat.of().formatHex(digest.digest());
  }

  private void checkWritable(final String path) {
    if (YumPaths.isInRepodata(path)) {
      throw new InvalidContentException("The repodata directories are generated by the repository: " + path);
    }
  }

  private void checkDepth(final String path) {
    if (layoutPolicy() == LayoutPolicy.STRICT && YumPaths.base(path, repodataDepth()).isEmpty()) {
      throw new InvalidContentException("RPM packages of this repository must be stored at least " + repodataDepth()
          + " directories deep, the depth at which its repodata is generated: " + path);
    }
  }

  private RpmPackage parse(final TempBlob blob) {
    try (InputStream in = blob.get()) {
      return RpmParser.parse(in);
    }
    catch (IOException e) {
      throw new InvalidContentException("Not a valid RPM package: " + e.getMessage());
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException("Not a valid RPM package: " + e.getMessage());
    }
  }

  private static boolean isMetadata(final FluentAsset asset) {
    return AssetKind.REPOMD.name().equals(asset.kind()) || AssetKind.REPODATA.name().equals(asset.kind());
  }

  private YumContentFacet content() {
    return facet(YumContentFacet.class);
  }
}
