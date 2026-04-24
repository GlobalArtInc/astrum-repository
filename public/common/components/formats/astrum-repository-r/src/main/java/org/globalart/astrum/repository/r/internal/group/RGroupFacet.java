package org.globalart.astrum.repository.r.internal.group;

import java.io.IOException;
import java.io.InputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.concurrent.locks.Lock;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.RepositoryCacheInvalidationService;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.group.GroupFacetImpl;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.r.RContentFacet;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.util.RPackagesUtils;
import org.globalart.astrum.repository.r.internal.util.RPackagesUtils.MergeResult;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import com.google.common.util.concurrent.Striped;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.r.RContentFacet.ATTRIBUTES_KEY;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.isGzip;

@Facet.Exposed
@Component
@Qualifier(RFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RGroupFacet
    extends GroupFacetImpl
{
  public static final String MERGE_KEY = "mergeKey";

  public static final String MEMBERS = "members";

  public static final String PACKAGE_COUNT = "packageCount";

  private final Striped<Lock> packagesLocks = Striped.lock(64);

  @Autowired
  public RGroupFacet(
      final RepositoryManager repositoryManager,
      final ConstraintViolationFactory constraintViolationFactory,
      @Qualifier(GroupType.NAME) final Type groupType,
      final RepositoryCacheInvalidationService repositoryCacheInvalidationService)
  {
    super(repositoryManager, constraintViolationFactory, groupType, repositoryCacheInvalidationService);
  }

  public Content mergedPackages(final String path, final Map<Repository, Content> memberPackages) throws IOException {
    if (memberPackages.size() == 1) {
      return memberPackages.values().iterator().next();
    }
    List<MemberPackages> members = new ArrayList<>();
    for (Entry<Repository, Content> entry : memberPackages.entrySet()) {
      members.add(new MemberPackages(entry.getKey().getName(), entry.getValue()));
    }
    String mergeKey = mergeKey(path, members);
    Optional<Content> cached = cached(path, mergeKey);
    if (cached.isPresent()) {
      return cached.get();
    }
    Lock lock = packagesLocks.get(path);
    lock.lock();
    try {
      cached = cached(path, mergeKey);
      if (cached.isPresent()) {
        return cached.get();
      }
      return merge(path, mergeKey, members);
    }
    finally {
      lock.unlock();
    }
  }

  private Content merge(final String path, final String mergeKey, final List<MemberPackages> members)
      throws IOException
  {
    List<String> texts = new ArrayList<>();
    List<String> merged = new ArrayList<>();
    for (MemberPackages member : members) {
      try (InputStream in = member.content.openInputStream()) {
        texts.add(RPackagesUtils.readPackages(in));
        merged.add(member.repositoryName);
      }
      catch (IOException e) {
        log.warn("Skipping unreadable {} from member {} of group {}: {}", path, member.repositoryName,
            getRepository().getName(), e.getMessage());
      }
    }
    MergeResult result = RPackagesUtils.merge(texts);
    boolean gzip = isGzip(path);
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(MERGE_KEY, mergeKey);
    attributes.put(MEMBERS, merged);
    attributes.put(PACKAGE_COUNT, result.packageCount());
    log.debug("Merged {} of group {} from {} with {} packages", path, getRepository().getName(), merged,
        result.packageCount());
    return content().putMetadata(path, RPackagesUtils.toBytes(result.packages(), gzip),
        gzip ? "application/x-gzip" : "text/plain", attributes);
  }

  private Optional<Content> cached(final String path, final String mergeKey) {
    Optional<FluentAsset> asset = content().getAsset(path);
    if (asset.isPresent() && mergeKey.equals(asset.get().attributes(ATTRIBUTES_KEY).get(MERGE_KEY, String.class))) {
      return Optional.of(asset.get().download());
    }
    return Optional.empty();
  }

  private static String mergeKey(final String path, final List<MemberPackages> members) throws IOException {
    MessageDigest digest = sha256();
    digest.update(path.getBytes(UTF_8));
    for (MemberPackages member : members) {
      digest.update(("\n" + member.repositoryName + "=" + member.digest()).getBytes(UTF_8));
    }
    return HexFormat.of().formatHex(digest.digest());
  }

  private static MessageDigest sha256() {
    try {
      return MessageDigest.getInstance("SHA-256");
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }

  private RContentFacet content() {
    return facet(RContentFacet.class);
  }

  private static final class MemberPackages
  {
    private final String repositoryName;

    private Content content;

    private MemberPackages(final String repositoryName, final Content content) {
      this.repositoryName = repositoryName;
      this.content = content;
    }

    private String digest() throws IOException {
      Asset asset = content.getAttributes().get(Asset.class);
      if (asset != null) {
        Optional<String> checksum = asset.blob().map(AssetBlob::checksums).map(checksums -> checksums.get("sha1"));
        if (checksum.isPresent()) {
          return checksum.get();
        }
      }
      byte[] bytes;
      try (InputStream in = content.openInputStream()) {
        bytes = in.readAllBytes();
      }
      content = new Content(new BytesPayload(bytes, content.getContentType()));
      return HexFormat.of().formatHex(sha256().digest(bytes));
    }
  }
}
