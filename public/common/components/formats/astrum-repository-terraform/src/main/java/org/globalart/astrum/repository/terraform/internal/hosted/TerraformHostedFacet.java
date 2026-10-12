package org.globalart.astrum.repository.terraform.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import java.util.TreeSet;
import java.util.concurrent.locks.Lock;
import java.util.function.Consumer;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.terraform.AssetKind;
import org.globalart.astrum.repository.terraform.ModuleArchives;
import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.Platform;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.ProviderFiles;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.terraform.internal.DownloadTokens;
import org.globalart.astrum.repository.terraform.internal.ModulePackages;
import org.globalart.astrum.repository.terraform.internal.SemanticVersions;
import org.globalart.astrum.repository.terraform.internal.TerraformContentTypes;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;
import org.globalart.astrum.repository.terraform.internal.signing.Pgp;
import org.globalart.astrum.repository.terraform.internal.signing.Sha256Sums;
import org.globalart.astrum.repository.terraform.internal.signing.TerraformSigningFacet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.google.common.util.concurrent.Striped;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.US_ASCII;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.ARCH;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.ATTRIBUTES_KEY;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.GENERATED;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.KEY_ID;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.OS;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.PROTOCOLS;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.SUBDIR;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.VERSION;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

/**
 * Modules and providers published to a hosted repository.
 *
 * A provider version is the set of release files of one {@code namespace/type/version}: a package per platform,
 * the {@code SHA256SUMS} document over the packages, its detached signature, optionally the release manifest that
 * names the plugin protocol versions, and the public key of the signature. Publishers may upload all of them as
 * their release tooling built them. When they upload no checksums, the repository writes them from the packages,
 * and when they upload no signature, it signs the checksums with its own key, if it has one.
 */
@Facet.Exposed
@Component
@Qualifier(TerraformFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TerraformHostedFacet
    extends FacetSupport
{
  public static final List<String> DEFAULT_PROTOCOLS = List.of("5.0");

  private final Striped<Lock> versionLocks = Striped.lock(64);

  public Content putModule(
      final ModuleCoordinates coordinates,
      @Nullable final String fileName,
      final Payload payload,
      final Consumer<String> authorizer) throws IOException
  {
    TerraformContentFacet content = content();
    try (TempBlob blob = content.ingest(payload)) {
      Optional<String> sniffed = ModuleArchives.sniff(head(blob));
      String extension = ModuleArchives.extension(fileName).orElse(sniffed.orElse(null));
      if (extension == null || sniffed.isEmpty() || !extension.equals(sniffed.get())) {
        throw new InvalidContentException("A module package must be a .zip or .tar.gz archive of the module directory");
      }
      String path = coordinates.archivePath(extension);
      authorizer.accept(path.substring(1));
      Lock lock = versionLocks.get(coordinates.directory());
      lock.lock();
      try {
        for (FluentAsset other : content.assetsIn(coordinates.directory(), AssetKind.MODULE_ARCHIVE)) {
          if (!other.path().equals(path)) {
            if (writePolicy() != WritePolicy.ALLOW) {
              throw new IllegalOperationException("Module " + coordinates.address() + " version "
                  + coordinates.version() + " already exists as " + other.path());
            }
            deleteAsset(other);
          }
        }
        String subdir = ModulePackages.subdirectory(blob, extension, null);
        return content.putModule(coordinates, path, blob, Map.of(SUBDIR, subdir)).download();
      }
      finally {
        lock.unlock();
      }
    }
  }

  public Optional<FluentAsset> moduleArchive(final ModuleCoordinates coordinates) {
    return content().assetsIn(coordinates.directory(), AssetKind.MODULE_ARCHIVE).stream().findFirst();
  }

  public List<String> moduleVersions(final ModuleCoordinates coordinates) {
    TreeSet<String> versions = new TreeSet<>(SemanticVersions.ORDER);
    for (FluentAsset asset : content().assetsUnder(coordinates.basePath(), AssetKind.MODULE_ARCHIVE)) {
      versionOf(asset, coordinates.basePath()).ifPresent(versions::add);
    }
    return new ArrayList<>(versions);
  }

  /**
   * Stores one release file of a provider version, recognized by its name, and brings the checksums and the
   * signature of the version up to date.
   */
  public Content putProviderFile(
      final ProviderCoordinates coordinates,
      final String fileName,
      final Payload payload,
      @Nullable final List<String> protocols,
      final Consumer<String> authorizer) throws IOException
  {
    AssetKind kind = ProviderFiles.kindOf(coordinates, fileName)
        .orElseGet(() -> ProviderFiles.isKeyUpload(fileName) ? AssetKind.PROVIDER_KEY : null);
    if (kind == null) {
      throw new InvalidContentException("'" + fileName + "' is not a release file of " + coordinates.address()
          + " " + coordinates.version() + "; expected " + coordinates.filePrefix() + "<os>_<arch>.zip, "
          + coordinates.shasumsFileName() + ", " + coordinates.signatureFileName() + ", "
          + coordinates.manifestFileName() + " or a public key (.asc, .gpg)");
    }
    String storedName = switch (kind) {
      case PROVIDER_SHASUMS -> coordinates.shasumsFileName();
      case PROVIDER_SIGNATURE -> coordinates.signatureFileName();
      case PROVIDER_MANIFEST -> coordinates.manifestFileName();
      case PROVIDER_KEY -> ProviderCoordinates.KEY_FILE_NAME;
      default -> fileName;
    };
    authorizer.accept(coordinates.filePath(storedName).substring(1));
    Lock lock = versionLocks.get(coordinates.directory());
    lock.lock();
    try {
      TerraformContentFacet content = content();
      Content stored;
      try (TempBlob blob = content.ingest(payload)) {
        stored = switch (kind) {
          case PROVIDER_ARCHIVE -> putPackage(coordinates, fileName, blob);
          case PROVIDER_SHASUMS -> putShasums(coordinates, blob);
          case PROVIDER_SIGNATURE -> putSignature(coordinates, blob);
          case PROVIDER_MANIFEST -> putManifest(coordinates, read(blob), false);
          case PROVIDER_KEY -> putKey(coordinates, blob);
          default -> throw new IllegalStateException(kind.name());
        };
      }
      if (protocols != null && !protocols.isEmpty() && kind != AssetKind.PROVIDER_MANIFEST
          && file(coordinates, coordinates.manifestFileName()).filter(asset -> !generated(asset)).isEmpty()) {
        putManifest(coordinates, manifest(protocols), true);
      }
      refreshChecksums(coordinates);
      return stored;
    }
    finally {
      lock.unlock();
    }
  }

  /**
   * Rewrites the checksums and the signature the repository generated for the versions of the given paths, after
   * packages were deleted.
   */
  public void refreshChecksums(final Collection<String> paths) {
    for (String path : new TreeSet<>(paths)) {
      versionDirectory(path).ifPresent(coordinates -> {
        Lock lock = versionLocks.get(coordinates.directory());
        lock.lock();
        try {
          refreshChecksums(coordinates);
        }
        catch (IOException | RuntimeException e) {
          log.warn("Unable to update the checksums of {} {} in repository {}", coordinates.address(),
              coordinates.version(), getRepository().getName(), e);
        }
        finally {
          lock.unlock();
        }
      });
    }
  }

  public Optional<ObjectNode> providerVersions(final ProviderCoordinates coordinates) {
    Map<String, TreeSet<Platform>> platforms = new TreeMap<>(SemanticVersions.ORDER);
    Map<String, List<String>> protocols = new LinkedHashMap<>();
    for (FluentAsset asset : content().assetsUnder(coordinates.basePath(), AssetKind.PROVIDER_ARCHIVE,
        AssetKind.PROVIDER_MANIFEST)) {
      Optional<String> version = versionOf(asset, coordinates.basePath());
      if (version.isEmpty()) {
        continue;
      }
      if (AssetKind.PROVIDER_ARCHIVE.name().equals(asset.kind())) {
        platformOf(asset).ifPresent(platform ->
            platforms.computeIfAbsent(version.get(), key -> new TreeSet<>()).add(platform));
      }
      else {
        protocols.put(version.get(), protocolsOf(asset));
      }
    }
    if (platforms.isEmpty()) {
      return Optional.empty();
    }
    ObjectNode document = TerraformResponses.object();
    ArrayNode versions = document.putArray("versions");
    platforms.forEach((version, available) -> {
      ObjectNode entry = versions.addObject();
      entry.put("version", version);
      ArrayNode protocolList = entry.putArray("protocols");
      protocols.getOrDefault(version, DEFAULT_PROTOCOLS).forEach(protocolList::add);
      ArrayNode platformList = entry.putArray("platforms");
      available.forEach(platform -> platformList.addObject().put("os", platform.os()).put("arch", platform.arch()));
    });
    return Optional.of(document);
  }

  /**
   * The package description Terraform installs a provider from, or why the version cannot be installed through the
   * registry protocol.
   */
  public ProviderPackage providerPackage(final ProviderCoordinates coordinates, final Platform platform)
      throws IOException
  {
    String archiveName = coordinates.archiveFileName(platform);
    Optional<FluentAsset> archive = file(coordinates, archiveName);
    if (archive.isEmpty()) {
      return ProviderPackage.missing(404, coordinates.address() + " " + coordinates.version()
          + " has no package for " + platform);
    }
    Optional<FluentAsset> sumsAsset = file(coordinates, coordinates.shasumsFileName());
    if (sumsAsset.isEmpty()) {
      return ProviderPackage.missing(404, "No checksums for " + coordinates.address() + " "
          + coordinates.version());
    }
    String shasum = Sha256Sums.parse(read(sumsAsset.get())).digest(archiveName).orElse(null);
    if (shasum == null) {
      return ProviderPackage.missing(404, coordinates.shasumsFileName() + " does not list " + archiveName);
    }
    Optional<FluentAsset> signature = file(coordinates, coordinates.signatureFileName());
    if (signature.isEmpty()) {
      return ProviderPackage.missing(404, coordinates.address() + " " + coordinates.version()
          + " is not signed: upload " + coordinates.signatureFileName()
          + " with its public key, or give the repository a signing key");
    }
    List<Pgp.PublicKey> keys = keys(coordinates, generated(signature.get()));
    if (keys.isEmpty()) {
      return ProviderPackage.missing(404, "No public key verifies the signature of " + coordinates.address()
          + " " + coordinates.version() + "; upload the key that signed " + coordinates.shasumsFileName());
    }
    String from = coordinates.downloadPath(platform);
    DownloadTokens tokens = DownloadTokens.of(content());
    ObjectNode document = TerraformResponses.object();
    ArrayNode protocols = document.putArray("protocols");
    file(coordinates, coordinates.manifestFileName()).map(TerraformHostedFacet::protocolsOf)
        .orElse(DEFAULT_PROTOCOLS).forEach(protocols::add);
    document.put("os", platform.os());
    document.put("arch", platform.arch());
    document.put("filename", archiveName);
    document.put("download_url", link(tokens, from, coordinates.filePath(archiveName)));
    document.put("shasums_url", link(tokens, from, coordinates.filePath(coordinates.shasumsFileName())));
    document.put("shasums_signature_url", link(tokens, from, coordinates.filePath(coordinates.signatureFileName())));
    document.put("shasum", shasum);
    ArrayNode publicKeys = document.putObject("signing_keys").putArray("gpg_public_keys");
    for (Pgp.PublicKey key : keys) {
      ObjectNode entry = publicKeys.addObject();
      entry.put("key_id", key.keyId());
      entry.put("ascii_armor", key.armored());
      entry.put("trust_signature", "");
      entry.put("source", getRepository().getName());
      entry.put("source_url", "");
    }
    return ProviderPackage.of(document);
  }

  public Optional<ObjectNode> mirrorIndex(final ProviderCoordinates coordinates) {
    return providerVersions(coordinates).map(versions -> {
      List<String> list = new ArrayList<>();
      versions.withArray("versions").forEach(entry -> list.add(entry.get("version").asText()));
      return TerraformResponses.mirrorIndex(list);
    });
  }

  /**
   * The packages of a provider version for the network mirror protocol, addressed relative to the document at
   * {@code documentPath}.
   */
  public Optional<ObjectNode> mirrorVersion(final ProviderCoordinates coordinates, final String documentPath) {
    List<FluentAsset> archives = content().assetsIn(coordinates.directory(), AssetKind.PROVIDER_ARCHIVE);
    if (archives.isEmpty()) {
      return Optional.empty();
    }
    DownloadTokens tokens = DownloadTokens.of(content());
    ObjectNode document = TerraformResponses.object();
    ObjectNode entries = document.putObject("archives");
    for (FluentAsset archive : archives) {
      Optional<Platform> platform = platformOf(archive);
      String sha256 = archive.blob().map(blob -> blob.checksums().get(HashAlgorithm.SHA256.name())).orElse(null);
      if (platform.isEmpty() || sha256 == null) {
        continue;
      }
      ObjectNode entry = entries.putObject(platform.get().toString());
      entry.put("url", link(tokens, documentPath, archive.path()));
      entry.putArray("hashes").add("zh:" + sha256);
    }
    return Optional.of(document);
  }

  private static String link(final DownloadTokens tokens, final String from, final String to) {
    return tokens.sign(TerraformPaths.relative(from, to), to);
  }

  private Content putPackage(final ProviderCoordinates coordinates, final String fileName, final TempBlob blob)
      throws IOException
  {
    if (!ModuleArchives.ZIP.equals(ModuleArchives.sniff(head(blob)).orElse(null))) {
      throw new InvalidContentException("A provider package must be a zip archive: " + fileName);
    }
    String sha256 = blob.getHashes().get(HashAlgorithm.SHA256).toString();
    Optional<FluentAsset> sums = file(coordinates, coordinates.shasumsFileName()).filter(asset -> !generated(asset));
    if (sums.isPresent()) {
      Optional<String> listed = Sha256Sums.parse(read(sums.get())).digest(fileName);
      if (listed.isEmpty()) {
        throw new InvalidContentException(coordinates.shasumsFileName() + " of this version does not list "
            + fileName + "; upload the release's checksums again with it");
      }
      if (!listed.get().equals(sha256)) {
        throw new InvalidContentException(fileName + " does not match its checksum in "
            + coordinates.shasumsFileName());
      }
    }
    Platform platform = ProviderFiles.platformOf(coordinates, fileName).orElseThrow();
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(OS, platform.os());
    attributes.put(ARCH, platform.arch());
    return content().putProviderFile(coordinates, fileName, AssetKind.PROVIDER_ARCHIVE, blob, attributes).download();
  }

  private Content putShasums(final ProviderCoordinates coordinates, final TempBlob blob) throws IOException {
    Sha256Sums sums;
    try {
      sums = Sha256Sums.parse(read(blob));
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException(e.getMessage());
    }
    for (FluentAsset archive : content().assetsIn(coordinates.directory(), AssetKind.PROVIDER_ARCHIVE)) {
      String name = fileName(archive);
      String sha256 = archive.blob().map(assetBlob -> assetBlob.checksums().get(HashAlgorithm.SHA256.name()))
          .orElse(null);
      Optional<String> listed = sums.digest(name);
      if (listed.isPresent() && !listed.get().equals(sha256)) {
        throw new InvalidContentException(coordinates.shasumsFileName() + " does not match the stored " + name);
      }
    }
    return content().putProviderFile(coordinates, coordinates.shasumsFileName(), AssetKind.PROVIDER_SHASUMS, blob,
        Map.of(GENERATED, false)).download();
  }

  private Content putSignature(final ProviderCoordinates coordinates, final TempBlob blob) throws IOException {
    byte[] signature;
    try {
      signature = Pgp.binarySignature(read(blob));
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException(e.getMessage());
    }
    Optional<FluentAsset> sums = file(coordinates, coordinates.shasumsFileName()).filter(asset -> !generated(asset));
    List<Pgp.PublicKey> keys = keys(coordinates, false);
    if (sums.isPresent() && !keys.isEmpty() && !verifies(read(sums.get()), signature, keys)) {
      throw new InvalidContentException(coordinates.signatureFileName() + " is not a signature of "
          + coordinates.shasumsFileName() + " by a known key");
    }
    return storeBytes(coordinates, coordinates.signatureFileName(), AssetKind.PROVIDER_SIGNATURE, signature,
        Map.of(GENERATED, false));
  }

  private Content putKey(final ProviderCoordinates coordinates, final TempBlob blob) throws IOException {
    Pgp.PublicKey key;
    try {
      key = Pgp.readPublicKey(read(blob));
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException(e.getMessage());
    }
    Optional<FluentAsset> sums = file(coordinates, coordinates.shasumsFileName()).filter(asset -> !generated(asset));
    Optional<FluentAsset> signature = file(coordinates, coordinates.signatureFileName())
        .filter(asset -> !generated(asset));
    if (sums.isPresent() && signature.isPresent()
        && !verifies(read(sums.get()), read(signature.get()), List.of(key))) {
      throw new InvalidContentException("Key " + key.keyId() + " did not sign " + coordinates.shasumsFileName());
    }
    return storeBytes(coordinates, ProviderCoordinates.KEY_FILE_NAME, AssetKind.PROVIDER_KEY,
        key.armored().getBytes(US_ASCII), Map.of(KEY_ID, key.keyId()));
  }

  private Content putManifest(final ProviderCoordinates coordinates, final byte[] manifest, final boolean generated)
      throws IOException
  {
    List<String> protocols = new ArrayList<>();
    try {
      JsonNode versions = TerraformResponses.read(manifest).path("metadata").path("protocol_versions");
      versions.forEach(version -> protocols.add(version.asText()));
    }
    catch (IOException e) {
      throw new InvalidContentException("The release manifest is not JSON: " + e.getMessage());
    }
    if (protocols.isEmpty()) {
      throw new InvalidContentException("The release manifest names no metadata.protocol_versions");
    }
    for (String protocol : protocols) {
      if (!protocol.matches("[0-9]+\\.[0-9]+")) {
        throw new InvalidContentException("Invalid plugin protocol version: " + protocol);
      }
    }
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(PROTOCOLS, protocols);
    attributes.put(GENERATED, generated);
    return storeBytes(coordinates, coordinates.manifestFileName(), AssetKind.PROVIDER_MANIFEST, manifest, attributes);
  }

  private void refreshChecksums(final ProviderCoordinates coordinates) throws IOException {
    List<FluentAsset> archives = content().assetsIn(coordinates.directory(), AssetKind.PROVIDER_ARCHIVE);
    Optional<FluentAsset> sums = file(coordinates, coordinates.shasumsFileName());
    Optional<FluentAsset> signature = file(coordinates, coordinates.signatureFileName());
    if (archives.isEmpty()) {
      sums.filter(TerraformHostedFacet::generated).ifPresent(this::deleteAsset);
      signature.filter(TerraformHostedFacet::generated).ifPresent(this::deleteAsset);
      return;
    }
    byte[] document;
    boolean changed = false;
    if (sums.isEmpty() || generated(sums.get())) {
      Map<String, String> digests = new TreeMap<>();
      for (FluentAsset archive : archives) {
        archive.blob().map(blob -> blob.checksums().get(HashAlgorithm.SHA256.name()))
            .ifPresent(sha256 -> digests.put(fileName(archive), sha256));
      }
      document = Sha256Sums.of(digests).toBytes();
      if (sums.isEmpty() || !Arrays.equals(document, read(sums.get()))) {
        storeBytes(coordinates, coordinates.shasumsFileName(), AssetKind.PROVIDER_SHASUMS, document,
            Map.of(GENERATED, true));
        changed = true;
      }
    }
    else {
      document = read(sums.get());
    }
    if (signature.isPresent() && !generated(signature.get())) {
      return;
    }
    Optional<Pgp.SigningKey> key = signing().flatMap(TerraformSigningFacet::signingKey);
    if (key.isEmpty()) {
      signature.ifPresent(this::deleteAsset);
      return;
    }
    String keyId = signature.map(asset -> asset.attributes(ATTRIBUTES_KEY).get(KEY_ID, String.class)).orElse(null);
    if (signature.isEmpty() || changed || !key.get().keyId().equals(keyId)) {
      storeBytes(coordinates, coordinates.signatureFileName(), AssetKind.PROVIDER_SIGNATURE,
          Pgp.sign(key.get(), document), Map.of(GENERATED, true, KEY_ID, key.get().keyId()));
    }
  }

  /**
   * The keys to list for a signature: the repository key for a signature the repository made, else the key the
   * publisher uploaded and the repository key, in case the publisher signed with it.
   */
  private List<Pgp.PublicKey> keys(final ProviderCoordinates coordinates, final boolean generatedSignature)
      throws IOException
  {
    List<Pgp.PublicKey> keys = new ArrayList<>();
    if (!generatedSignature) {
      Optional<FluentAsset> uploaded = file(coordinates, ProviderCoordinates.KEY_FILE_NAME);
      if (uploaded.isPresent()) {
        keys.add(Pgp.readPublicKey(read(uploaded.get())));
      }
    }
    signing().flatMap(TerraformSigningFacet::signingKey)
        .map(key -> new Pgp.PublicKey(key.keyId(), key.armoredPublicKey()))
        .filter(key -> keys.stream().noneMatch(known -> known.keyId().equals(key.keyId())))
        .ifPresent(keys::add);
    return keys;
  }

  private static boolean verifies(final byte[] document, final byte[] signature, final List<Pgp.PublicKey> keys) {
    return keys.stream().anyMatch(key -> Pgp.verify(document, signature, key.armored()));
  }

  private Content storeBytes(
      final ProviderCoordinates coordinates,
      final String fileName,
      final AssetKind kind,
      final byte[] bytes,
      final Map<String, Object> attributes)
  {
    String contentType = switch (kind) {
      case PROVIDER_SHASUMS -> TEXT_PLAIN;
      case PROVIDER_MANIFEST -> APPLICATION_JSON;
      default -> TerraformContentTypes.forKind(kind.name());
    };
    try (TempBlob blob = content().ingest(new BytesPayload(bytes, contentType))) {
      return content().putProviderFile(coordinates, fileName, kind, blob, attributes).download();
    }
  }

  private Optional<FluentAsset> file(final ProviderCoordinates coordinates, final String fileName) {
    return content().getAsset(coordinates.filePath(fileName));
  }

  private static byte[] manifest(final List<String> protocols) {
    ObjectNode manifest = TerraformResponses.object();
    manifest.put("version", 1);
    ArrayNode versions = manifest.putObject("metadata").putArray("protocol_versions");
    protocols.forEach(versions::add);
    return TerraformResponses.bytes(manifest);
  }

  private static boolean generated(final FluentAsset asset) {
    return Boolean.TRUE.equals(asset.attributes(ATTRIBUTES_KEY).get(GENERATED));
  }

  private static List<String> protocolsOf(final FluentAsset asset) {
    Object value = asset.attributes(ATTRIBUTES_KEY).get(PROTOCOLS);
    if (value instanceof Collection<?> list && !list.isEmpty()) {
      return list.stream().map(String::valueOf).toList();
    }
    return DEFAULT_PROTOCOLS;
  }

  private static Optional<Platform> platformOf(final FluentAsset asset) {
    Object os = asset.attributes(ATTRIBUTES_KEY).get(OS);
    Object arch = asset.attributes(ATTRIBUTES_KEY).get(ARCH);
    if (os instanceof String osName && arch instanceof String archName
        && TerraformPaths.isValidPlatformPart(osName) && TerraformPaths.isValidPlatformPart(archName)) {
      return Optional.of(new Platform(osName, archName));
    }
    return Optional.empty();
  }

  private static Optional<String> versionOf(final FluentAsset asset, final String basePath) {
    Object version = asset.attributes(ATTRIBUTES_KEY).get(VERSION);
    if (version instanceof String value && TerraformPaths.isValidVersion(value)) {
      return Optional.of(value);
    }
    String rest = asset.path().substring(basePath.length() + 1);
    int slash = rest.indexOf('/');
    String segment = slash < 0 ? rest : rest.substring(0, slash);
    return TerraformPaths.isValidVersion(segment) ? Optional.of(segment) : Optional.empty();
  }

  /**
   * The provider version whose directory holds the given path, for paths of provider packages.
   */
  static Optional<ProviderCoordinates> versionDirectory(final String path) {
    String normalized = path.startsWith("/") ? path : "/" + path;
    if (!normalized.startsWith(TerraformPaths.PROVIDERS)) {
      return Optional.empty();
    }
    String[] segments = normalized.substring(TerraformPaths.PROVIDERS.length()).split("/");
    if (segments.length != 4 || !TerraformPaths.isValidName(segments[0]) || !TerraformPaths.isValidName(segments[1])
        || !TerraformPaths.isValidVersion(segments[2])) {
      return Optional.empty();
    }
    ProviderCoordinates coordinates = new ProviderCoordinates(segments[0], segments[1], segments[2]);
    return ProviderFiles.platformOf(coordinates, segments[3]).map(platform -> coordinates);
  }

  private static String fileName(final FluentAsset asset) {
    return asset.path().substring(asset.path().lastIndexOf('/') + 1);
  }

  private static byte[] head(final TempBlob blob) throws IOException {
    try (InputStream in = blob.get()) {
      return in.readNBytes(8);
    }
  }

  private static byte[] read(final TempBlob blob) throws IOException {
    try (InputStream in = blob.get()) {
      return in.readAllBytes();
    }
  }

  private static byte[] read(final FluentAsset asset) throws IOException {
    try (InputStream in = asset.download().openInputStream()) {
      return in.readAllBytes();
    }
  }

  private WritePolicy writePolicy() {
    Object value = getRepository().getConfiguration().attributes("storage").get("writePolicy");
    if (value == null) {
      return WritePolicy.ALLOW;
    }
    try {
      return WritePolicy.valueOf(value.toString());
    }
    catch (IllegalArgumentException e) {
      return WritePolicy.ALLOW_ONCE;
    }
  }

  private void deleteAsset(final FluentAsset asset) {
    facet(ContentMaintenanceFacet.class).deleteAsset(asset);
  }

  private Optional<TerraformSigningFacet> signing() {
    return getRepository().optionalFacet(TerraformSigningFacet.class);
  }

  private TerraformContentFacet content() {
    return facet(TerraformContentFacet.class);
  }

  /**
   * The answer of the provider registry for one package: the document, or the status and reason it is missing.
   */
  public record ProviderPackage(@Nullable ObjectNode document, int status, @Nullable String reason)
  {
    static ProviderPackage of(final ObjectNode document) {
      return new ProviderPackage(document, 200, null);
    }

    static ProviderPackage missing(final int status, final String reason) {
      return new ProviderPackage(null, status, reason);
    }
  }
}
